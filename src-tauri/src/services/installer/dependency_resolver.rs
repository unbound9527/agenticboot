//! 依赖解析器
//!
//! 通过拓扑排序构建安装计划，自动处理传递依赖。
//! 同一依赖被多个工具需要时只安装一次。

use crate::plugin::get_plugin_by_id;
use crate::tool_types::{InstallPlan, InstallStep};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

/// 解析工具安装计划
///
/// # Arguments
/// * `tool_ids` - 用户选择的工具 ID 列表
/// * `install_root` - 自定义安装根目录，用于检测已有安装
///
/// # Returns
/// 安装计划（按依赖顺序排列，依赖在前，目标工具在后）
pub fn resolve_install_plan(tool_ids: &[String], install_root: Option<&Path>) -> Result<InstallPlan, String> {
    // 第一步：收集所有被请求的工具及其传递依赖
    let mut all_ids = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::from(tool_ids.to_vec());

    while let Some(id) = queue.pop_front() {
        if all_ids.insert(id.clone()) {
            if let Some(plugin) = get_plugin_by_id(&id) {
                for dep in plugin.dependencies() {
                    queue.push_back(dep.tool_id);
                }
            } else {
                return Err(format!("未知工具: {id}"));
            }
        }
    }

    // 第二步：构建依赖图（A → B 表示 A 依赖 B，B 必须先安装）
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for id in &all_ids {
        if let Some(plugin) = get_plugin_by_id(id) {
            let deps: Vec<String> = plugin.dependencies().into_iter().map(|d| d.tool_id).collect();
            graph.insert(id.clone(), deps);
        }
    }

    // 第三步：拓扑排序（Kahn 算法）
    let sorted = topological_sort(&graph)?;

    // 第四步：构建安装步骤
    let original_ids: HashSet<&String> = tool_ids.iter().collect();
    let mut steps = Vec::new();

    for id in &sorted {
        let plugin = get_plugin_by_id(id).ok_or_else(|| format!("未知工具: {id}"))?;
        let meta = plugin.metadata();
        let detect = plugin.detect(install_root);

        let reason = if original_ids.contains(id) {
            "selected".to_string()
        } else {
            // 查找哪些已选择的工具依赖此工具
            let dependents: Vec<&String> = original_ids
                .iter()
                .filter(|tid| {
                    if let Some(p) = get_plugin_by_id(tid) {
                        p.dependencies().iter().any(|d| &d.tool_id == id)
                    } else {
                        false
                    }
                })
                .copied()
                .collect();
            let parent = dependents.first().map(|s| s.as_str()).unwrap_or("unknown");
            format!("dependency_of({parent})")
        };

        steps.push(InstallStep {
            tool_id: id.clone(),
            tool_name: meta.name,
            category: meta.category,
            reason,
            is_installed: detect.installed,
        });
    }

    Ok(InstallPlan { steps })
}

/// Kahn 算法拓扑排序
fn topological_sort(graph: &HashMap<String, Vec<String>>) -> Result<Vec<String>, String> {
    // 计算入度
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    for node in graph.keys() {
        in_degree.entry(node.clone()).or_insert(0);
    }
    for deps in graph.values() {
        for dep in deps {
            *in_degree.entry(dep.clone()).or_insert(0) += 1;
        }
    }

    // 入度为 0 的节点入队
    let mut queue: VecDeque<String> = VecDeque::new();
    for (node, degree) in &in_degree {
        if *degree == 0 {
            queue.push_back(node.clone());
        }
    }

    let mut sorted = Vec::new();
    while let Some(node) = queue.pop_front() {
        sorted.push(node.clone());
        if let Some(deps) = graph.get(&node) {
            for dep in deps {
                if let Some(degree) = in_degree.get_mut(dep) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(dep.clone());
                    }
                }
            }
        }
    }

    // 循环依赖检测
    if sorted.len() != graph.len() {
        return Err("检测到循环依赖，请检查插件依赖声明".to_string());
    }

    // 反转结果：拓扑排序得到的是被依赖者在前，我们需要依赖项在前
    sorted.reverse();
    Ok(sorted)
}
