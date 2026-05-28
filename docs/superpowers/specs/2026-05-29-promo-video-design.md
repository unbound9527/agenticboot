# AgenticBoot 宣传视频 — 设计规格

## 概述

- **目标受众**：普通用户（非技术背景）
- **视频时长**：约 2 分 30 秒
- **视觉风格**：温暖亲民风（暖橙 + 浅金 + 暖白底）
- **语言**：纯中文
- **技术方案**：React + Framer Motion + Tailwind CSS → Playwright 录制 → ffmpeg 合成 MP4
- **工作目录**：`/.claude/promo-video/`（gitignored）

---

## 技术架构

```
src/
├── App.tsx                  # 总控：时间线编排 + 场景切换
├── main.tsx                 # Vite 入口
├── index.css                # Tailwind + 全局样式 + 自定义动画
├── scenes/
│   ├── Scene00Cover.tsx      # 封面片头
│   ├── Scene01Doubao.tsx     # 豆包 vs 桌面AI 数据对比
│   ├── Scene02Reality.tsx    # 现实落差
│   ├── Scene03Vision.tsx     # 愿景
│   ├── Scene04WhatIs.tsx     # AgenticBoot 是什么
│   ├── Scene05Capabilities.tsx # 四大核心能力
│   ├── Scene06Demo.tsx       # 操作演示
│   ├── Scene07CCSwitch.tsx   # 为什么基于 CC Switch
│   ├── Scene08Roadmap.tsx    # 未来路线图
│   └── Scene09CTA.tsx        # 结尾 CTA
├── components/
│   ├── Blob.tsx              # 背景呼吸 blob
│   ├── AnimatedCounter.tsx   # 数字跳动
│   ├── BarChart.tsx          # 柱状对比图
│   ├── Timeline.tsx          # 时间线/路线图
│   ├── Card.tsx              # 能力卡片
│   └── FlowArrow.tsx         # 流程箭头
├── hooks/
│   └── useSceneTimer.ts      # 场景时长控制
├── record.ts                 # Playwright 录制脚本
└── compose.sh               # ffmpeg 合成脚本
```

**依赖**：
- `react` + `react-dom`（已有）
- `framer-motion`（已有）
- `tailwindcss`（已有）
- `lucide-react`（已有，图标）
- `playwright`（新增，仅录制用）
- `ffmpeg`（系统安装，仅合成用）

---

## 全局视觉系统

### 配色

| 用途 | 色值 | Tailwind |
|------|------|----------|
| 主色 | `#FF6B35` | `brand-orange` |
| 辅助 | `#FFB563` | `brand-gold` |
| 强调 | `#26C9A5` | `brand-mint` |
| 背景 | `#FFFAF5` | `brand-warm` |
| 深色文字 | `#2D3436` | `brand-charcoal` |
| 卡片底色 | `#FFFFFF` | white |

### 动效规范

- **页面过渡**：`framer-motion` AnimatePresence，淡入 + 上移 20px，spring stiffness:80 damping:20
- **卡片入场**：staggerChildren: 0.08s，从下方浮入
- **数字跳动**：useSpring + interpolate，count-up 效果
- **图标强调**：scale bounce（1→1.3→1），配合浅色 glow
- **背景 blob**：absolute 定位，柔和渐变，scale 和位置缓慢循环变化（15s 周期）

### 字体

- 系统默认中文字体栈：`"PingFang SC", "Microsoft YaHei", "Noto Sans SC", sans-serif`
- 标题字重：`font-bold`（700）
- 正文：`font-normal`（400）

---

## 场景详情

### 场景 0 · 封面片头（5s）

- 暖橙→浅金渐变背景 + 居中发光 blob（scale pulse 呼吸）
- Logo 从模糊到清晰（filter: blur → none + scale 0.9→1.0）
- "AgenticBoot" 标题从下方浮入（y: 30→0）
- "AI 工具，一点就装" 副标题延迟 0.3s 淡入
- 底部小箭头轻微弹跳（y 轴 loop）

### 场景 1 · 豆包 vs 桌面 AI 工具（20s）

**子节拍 1（0-3s）**：豆包/通义千问/Kimi 图标卡片，文字"它们让 AI 走进了日常生活"

**子节拍 2（3-16s）**：三组柱状对比图 stagger 入场

| 场景 | 仅用豆包 | 搭配桌面 AI 工具 | 效率提升 |
|------|----------|------------------|----------|
| 整理 100 张照片 | 40 分钟 | 3 分钟 | 92% |
| 处理 50 份文档/表格 | 60 分钟 | 5 分钟 | 92% |
| 搭建本地自动化工作流 | 0%（做不到） | 100%（直接操作） | — |

视觉：左右两栏，豆包侧柱状矮、低饱和度；桌面 AI 侧柱状高、暖橙亮色。第三组用 ❌/✔️ 对比替代柱状图。

**子节拍 3（16-20s）**：结论大字"效率平均提升 90%+" + glow

### 场景 2 · 现实落差（15s）

**子节拍 1（0-8s）**：AI 工具图标前方，障碍墙依次砸下——"终端命令行"、"环境变量"、"PATH 配置"、"Node.js 版本"——向下 slide + shake

**子节拍 2（8-15s）**：人物剪影 + 安装文档堆 + 大问号。"还没见到 AI，先被安装劝退"。色调偏冷灰。

### 场景 3 · 愿景（12s）

- 暖橙背景回归
- "让所有人" → "不管懂不懂技术" → "都能便捷使用 AI 工具" 逐段浮现
- 底部细线从左到右延伸，象征门槛被拉平
- 背景 blob 变大变亮

### 场景 4 · AgenticBoot 是什么（13s）

- 居中流程图：`[🔍 检测] → [📦 补齐] → [📋 管理]`
- 箭头以 SVG stroke-dashoffset 动画画出
- 底部："一个入口，搞定 AI 工具的安装与管理"

### 场景 5 · 四大核心能力（25s）

四张卡片 stagger 入场，每张约 6s：

1. **智能检测**：扫描线扫过列表，已装项打勾
2. **一键安装**：进度条 0→100%，完成弹绿勾
3. **统一管理**：多图标聚合到面板
4. **过程透明**：终端日志逐行滚动

每张卡片：左图标+动效 / 右标题+描述

### 场景 6 · 操作演示（25s）

- 前半（12s）：Wizard 装机向导 mockup——步骤条动画
- 后半（13s）：Manager 软件管家 mockup——已装列表滚动
- 带桌面窗口边框（圆角 + 阴影）

### 场景 7 · 为什么基于 CC Switch（20s）

- 左：CC Switch 图标 + "成熟桌面框架 + 跨平台基础"
- 右：AgenticBoot 图标 + "装机检测 · 工具管理 · 体验创新"
- 中间箭头从左"生长"到右
- 文字："站在成熟的肩膀上，把精力全部投入装机体验"

### 场景 8 · 未来路线图（10s）

- 横轴时间线：✅ 已完成 Windows → ○ 进行中（更多工具 + 打磨）→ ◌ 计划中（macOS/Linux）
- 时间线从左到右动画画出
- "这只是开始"

### 场景 9 · 结尾 CTA（5s）

- 暖橙背景呼应封面
- Logo + "让 AI 工具触手可及"
- GitHub 地址 + Star 按钮 + 二维码占位

---

## 录制与合成

### Playwright 录制脚本

```
record.ts 流程：
1. 启动 Vite dev server
2. 打开无头浏览器，1920×1080 全屏
3. 注入 CSS 隐藏滚动条
4. MediaRecorder API 或 page.screenshot() 逐帧采集
5. 输出 raw-frames/ 目录 + 音频静默
```

### ffmpeg 合成

```bash
ffmpeg -framerate 30 -i raw-frames/frame-%05d.png \
       -c:v libx264 -preset slow -crf 18 \
       -pix_fmt yuv420p \
       output/agenticboot-promo.mp4
```

帧率：30fps，分辨率：1920×1080

### 可选增强
- 背景音乐：温暖轻快的电子/原声（需另找素材）
- AI 配音旁白：可以后期追加音轨

---

## 文件结构（在 /.claude/promo-video/ 下）

```
.claude/promo-video/
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tailwind.config.js
├── postcss.config.js
├── index.html
├── src/               # 见上文 src/ 结构
├── scripts/
│   ├── record.ts      # Playwright 录制
│   └── compose.sh     # ffmpeg 合成
└── output/            # 产物目录
    └── agenticboot-promo.mp4
```
