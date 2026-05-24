//! 依赖解析器
//!
//! 通过拓扑排序构建安装计划，自动处理传递依赖。
//! 同一依赖被多个工具需要时只安装一次。

use crate::plugin::get_plugin_by_id;
use crate::tool_types::{InstallPlan, InstallStep};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

fn satisfies_min_version(current: &str, requirement: &str) -> bool {
    let req = requirement.trim();
    if let Some(version_str) = req.strip_prefix(">=") {
        let min_ver = version_str.trim();
        return compare_semver(current, min_ver) >= 0;
    }
    if let Some(version_str) = req.strip_prefix(">") {
        let min_ver = version_str.trim();
        return compare_semver(current, min_ver) > 0;
    }
    if let Some(version_str) = req.strip_prefix("<=") {
        let min_ver = version_str.trim();
        return compare_semver(current, min_ver) <= 0;
    }
    if let Some(version_str) = req.strip_prefix("<") {
        let min_ver = version_str.trim();
        return compare_semver(current, min_ver) < 0;
    }
    compare_semver(current, req) == 0
}

fn compare_semver(a: &str, b: &str) -> i32 {
    let a_parts: Vec<u64> = a
        .trim_start_matches('v')
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let b_parts: Vec<u64> = b
        .trim_start_matches('v')
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let max_len = a_parts.len().max(b_parts.len());
    for i in 0..max_len {
        let av = a_parts.get(i).copied().unwrap_or(0);
        let bv = b_parts.get(i).copied().unwrap_or(0);
        match av.cmp(&bv) {
            std::cmp::Ordering::Less => return -1,
            std::cmp::Ordering::Greater => return 1,
            std::cmp::Ordering::Equal => continue,
        }
    }
    0
}

fn dependencies_satisfied(tool_id: &str, install_root: Option<&Path>) -> bool {
    let Some(plugin) = get_plugin_by_id(tool_id) else {
        return false;
    };

    for dependency in plugin.dependencies() {
        let Some(dep_plugin) = get_plugin_by_id(&dependency.tool_id) else {
            return false;
        };
        let dep_detect = dep_plugin.detect(install_root);
        if !dep_detect.installed {
            return false;
        }
        if let Some(min_version) = dependency.min_version.as_deref() {
            let Some(dep_version) = dep_detect.version.as_deref() else {
                return false;
            };
            if !satisfies_min_version(dep_version, min_version) {
                return false;
            }
        }
    }

    true
}

fn dependent_requirements_satisfied(
    tool_id: &str,
    requested_tool_ids: &HashSet<String>,
    install_root: Option<&Path>,
) -> bool {
    let Some(plugin) = get_plugin_by_id(tool_id) else {
        return false;
    };

    let detect = plugin.detect(install_root);
    if !detect.installed {
        return false;
    }

    for requested_tool_id in requested_tool_ids {
        let Some(requested_plugin) = get_plugin_by_id(requested_tool_id) else {
            return false;
        };

        for dependency in requested_plugin.dependencies() {
            if dependency.tool_id != tool_id {
                continue;
            }

            if let Some(min_version) = dependency.min_version.as_deref() {
                let Some(current_version) = detect.version.as_deref() else {
                    return false;
                };
                if !satisfies_min_version(current_version, min_version) {
                    return false;
                }
            }
        }
    }

    true
}

/// 解析工具安装计划
///
/// # Arguments
/// * `tool_ids` - 用户选择的工具 ID 列表
/// * `install_root` - 自定义安装根目录，用于检测已有安装
///
/// # Returns
/// 安装计划（按依赖顺序排列，依赖在前，目标工具在后）
pub fn resolve_install_plan(
    tool_ids: &[String],
    install_root: Option<&Path>,
) -> Result<InstallPlan, String> {
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
            let deps: Vec<String> = plugin
                .dependencies()
                .into_iter()
                .map(|d| d.tool_id)
                .collect();
            graph.insert(id.clone(), deps);
        }
    }

    // 第三步：拓扑排序（Kahn 算法）
    let sorted = topological_sort(&graph)?;

    // 第四步：构建安装步骤
    let original_ids: HashSet<&String> = tool_ids.iter().collect();
    let all_ids_owned: HashSet<String> = all_ids.iter().cloned().collect();
    let mut steps = Vec::new();

    for id in &sorted {
        let plugin = get_plugin_by_id(id).ok_or_else(|| format!("未知工具: {id}"))?;
        let meta = plugin.metadata();
        let detect = plugin.detect(install_root);

        let is_installed = detect.installed
            && dependencies_satisfied(id, install_root)
            && dependent_requirements_satisfied(id, &all_ids_owned, install_root);

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
            is_installed,
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

#[cfg(test)]
mod tests {
    use super::{compare_semver, resolve_install_plan, satisfies_min_version};

    #[test]
    fn compare_semver_ignores_v_prefix() {
        assert_eq!(compare_semver("v20.20.2", "20.20.2"), 0);
    }

    #[test]
    fn satisfies_min_version_accepts_newer_patch_versions() {
        assert!(satisfies_min_version("20.20.2", ">= 18.0.0"));
    }

    #[test]
    fn resolve_install_plan_treats_gemini_as_installed_when_node_dependency_is_satisfied() {
        let tmp = tempfile::tempdir().unwrap();
        let gemini_dir = tmp.path().join("gemini-cli");

        std::fs::create_dir_all(&gemini_dir).unwrap();
        std::fs::write(
            gemini_dir.join("gemini.cmd"),
            "@echo off\r\necho 0.41.2\r\n",
        )
        .unwrap();

        let plan =
            resolve_install_plan(&["gemini-cli".to_string()], Some(tmp.path())).expect("plan");

        let gemini_step = plan
            .steps
            .iter()
            .find(|step| step.tool_id == "gemini-cli")
            .expect("gemini step");
        assert!(
            gemini_step.is_installed,
            "gemini should remain installed when node dependency is already satisfied"
        );
    }

    #[test]
    fn resolve_install_plan_marks_nodejs_for_install_when_dependent_needs_newer_version() {
        let tmp = tempfile::tempdir().unwrap();
        let node_dir = tmp.path().join("nodejs");

        std::fs::create_dir_all(&node_dir).unwrap();
        std::fs::write(node_dir.join("node.exe"), "@echo off\r\necho v20.20.2\r\n").unwrap();

        let plan = resolve_install_plan(&["openclaw".to_string()], Some(tmp.path())).expect("plan");

        let node_step = plan
            .steps
            .iter()
            .find(|step| step.tool_id == "nodejs")
            .expect("node step");
        assert!(
            !node_step.is_installed,
            "nodejs should be scheduled because openclaw requires a newer node version"
        );
    }

    #[test]
    fn resolve_install_plan_does_not_require_git_for_openclaw() {
        let plan = resolve_install_plan(&["openclaw".to_string()], None).expect("plan");

        assert!(plan.steps.iter().any(|step| step.tool_id == "openclaw"));
        assert!(
            !plan.steps.iter().any(|step| step.tool_id == "git"),
            "openclaw should no longer pull git in as a direct dependency"
        );
    }
}
