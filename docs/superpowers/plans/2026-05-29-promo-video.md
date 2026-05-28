# AgenticBoot 宣传视频 — 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 构建一个 2 分 30 秒的 AgenticBoot 宣传视频，基于 React + Framer Motion 动画幻灯片，通过 Playwright 录制 + ffmpeg 合成为 MP4。

**Architecture:** React SPA 包含 10 个场景组件，App.tsx 通过定时器驱动场景切换，每个场景内部用 Framer Motion 编排入场/出场动画。共享组件（Blob、BarChart、Timeline 等）在各场景中复用。录制脚本用 Playwright 截帧，ffmpeg 合成为 30fps 视频。

**Tech Stack:** React 18, TypeScript, Tailwind CSS 3, Framer Motion 12, Vite 7, Playwright, ffmpeg

**工作目录:** `/.claude/promo-video/`（已 gitignored）

---

## 文件结构总览

```
.claude/promo-video/
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tailwind.config.js
├── postcss.config.js
├── index.html
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── index.css
│   ├── scenes/
│   │   ├── Scene00Cover.tsx
│   │   ├── Scene01Doubao.tsx
│   │   ├── Scene02Reality.tsx
│   │   ├── Scene03Vision.tsx
│   │   ├── Scene04WhatIs.tsx
│   │   ├── Scene05Capabilities.tsx
│   │   ├── Scene06Demo.tsx
│   │   ├── Scene07CCSwitch.tsx
│   │   ├── Scene08Roadmap.tsx
│   │   └── Scene09CTA.tsx
│   ├── components/
│   │   ├── Blob.tsx
│   │   ├── AnimatedCounter.tsx
│   │   ├── BarChart.tsx
│   │   ├── Timeline.tsx
│   │   ├── Card.tsx
│   │   ├── FlowArrow.tsx
│   │   └── SceneWrapper.tsx
│   └── data/
│       └── timeline.ts       # 场景时长配置
├── scripts/
│   ├── record.ts             # Playwright 录制
│   └── compose.sh            # ffmpeg 合成
└── output/                   # 产物目录
```

---

### Task 0: 项目脚手架

**Files:**
- Create: `/.claude/promo-video/package.json`
- Create: `/.claude/promo-video/vite.config.ts`
- Create: `/.claude/promo-video/tsconfig.json`
- Create: `/.claude/promo-video/tailwind.config.js`
- Create: `/.claude/promo-video/postcss.config.js`
- Create: `/.claude/promo-video/index.html`

- [ ] **Step 1: 创建 package.json**

```json
{
  "name": "agenticboot-promo",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "record": "npx tsx scripts/record.ts",
    "compose": "bash scripts/compose.sh"
  },
  "dependencies": {
    "framer-motion": "^12.23.25",
    "lucide-react": "^0.542.0",
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  },
  "devDependencies": {
    "@types/react": "^18.2.0",
    "@types/react-dom": "^18.2.0",
    "autoprefixer": "^10.4.20",
    "postcss": "^8.4.49",
    "playwright": "^1.52.0",
    "tailwindcss": "^3.4.17",
    "typescript": "^5.3.0",
    "vite": "^7.3.0"
  }
}
```

- [ ] **Step 2: 创建 vite.config.ts**

```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  root: ".",
  build: { outDir: "dist" },
});
```

- [ ] **Step 3: 创建 tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "noEmit": true,
    "forceConsistentCasingInFileNames": true
  },
  "include": ["src"]
}
```

- [ ] **Step 4: 创建 tailwind.config.js**

```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        brand: {
          orange: "#FF6B35",
          gold: "#FFB563",
          mint: "#26C9A5",
          warm: "#FFFAF5",
          charcoal: "#2D3436",
        },
      },
      fontFamily: {
        sans: [
          "PingFang SC",
          "Microsoft YaHei",
          "Noto Sans SC",
          "sans-serif",
        ],
      },
    },
  },
  plugins: [],
};
```

- [ ] **Step 5: 创建 postcss.config.js**

```javascript
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
};
```

- [ ] **Step 6: 创建 index.html**

```html
<!DOCTYPE html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>AgenticBoot - AI 工具，一点就装</title>
    <style>
      * { margin: 0; padding: 0; box-sizing: border-box; }
      html, body, #root { width: 100%; height: 100%; overflow: hidden; }
      body { background: #FFFAF5; }
    </style>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

- [ ] **Step 7: 安装依赖**

```bash
cd /.claude/promo-video && npm install
```

- [ ] **Step 8: Commit**

```bash
git add .claude/promo-video/package.json .claude/promo-video/vite.config.ts .claude/promo-video/tsconfig.json .claude/promo-video/tailwind.config.js .claude/promo-video/postcss.config.js .claude/promo-video/index.html
git commit -m "feat: scaffold promo-video project with Vite + React + Tailwind"
```

---

### Task 1: 全局样式 + 入口文件

**Files:**
- Create: `/.claude/promo-video/src/index.css`
- Create: `/.claude/promo-video/src/main.tsx`

- [ ] **Step 1: 创建 index.css**

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    --color-orange: #FF6B35;
    --color-gold: #FFB563;
    --color-mint: #26C9A5;
    --color-warm: #FFFAF5;
    --color-charcoal: #2D3436;
  }

  body {
    font-family: "PingFang SC", "Microsoft YaHei", "Noto Sans SC", sans-serif;
    color: #2D3436;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  /* 隐藏滚动条，视频不需要 */
  *::-webkit-scrollbar {
    display: none;
  }
  * {
    -ms-overflow-style: none;
    scrollbar-width: none;
  }
}

@layer components {
  .scene-container {
    @apply absolute inset-0 flex flex-col items-center justify-center;
    width: 1920px;
    height: 1080px;
    overflow: hidden;
  }
}
```

- [ ] **Step 2: 创建 main.tsx**

```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
```

- [ ] **Step 3: Commit**

```bash
git add .claude/promo-video/src/index.css .claude/promo-video/src/main.tsx
git commit -m "feat: add global styles and React entry point"
```

---

### Task 2: 共享组件 — Blob + SceneWrapper

**Files:**
- Create: `/.claude/promo-video/src/components/Blob.tsx`
- Create: `/.claude/promo-video/src/components/SceneWrapper.tsx`

- [ ] **Step 1: 创建 Blob.tsx**

```tsx
import { motion } from "framer-motion";

interface BlobProps {
  color?: string;
  size?: number;
  opacity?: number;
  top?: string;
  left?: string;
  duration?: number;
}

export default function Blob({
  color = "#FFB563",
  size = 600,
  opacity = 0.3,
  top = "50%",
  left = "50%",
  duration = 15,
}: BlobProps) {
  return (
    <motion.div
      className="absolute rounded-full blur-3xl pointer-events-none"
      style={{
        width: size,
        height: size,
        background: color,
        opacity,
        top,
        left,
        transform: "translate(-50%, -50%)",
      }}
      animate={{
        scale: [1, 1.15, 1],
        x: ["-50%", "-45%", "-50%"],
        y: ["-50%", "-55%", "-50%"],
      }}
      transition={{
        duration,
        repeat: Infinity,
        ease: "easeInOut",
      }}
    />
  );
}
```

- [ ] **Step 2: 创建 SceneWrapper.tsx**

```tsx
import { motion, AnimatePresence } from "framer-motion";
import { ReactNode } from "react";

interface SceneWrapperProps {
  show: boolean;
  children: ReactNode;
}

export default function SceneWrapper({ show, children }: SceneWrapperProps) {
  return (
    <AnimatePresence>
      {show && (
        <motion.div
          className="scene-container"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -20 }}
          transition={{ duration: 0.6, ease: [0.25, 0.46, 0.45, 0.94] }}
        >
          {children}
        </motion.div>
      )}
    </AnimatePresence>
  );
}
```

- [ ] **Step 3: Commit**

```bash
git add .claude/promo-video/src/components/Blob.tsx .claude/promo-video/src/components/SceneWrapper.tsx
git commit -m "feat: add Blob and SceneWrapper shared components"
```

---

### Task 3: 场景时长数据 + App 总控

**Files:**
- Create: `/.claude/promo-video/src/data/timeline.ts`
- Create: `/.claude/promo-video/src/App.tsx`

- [ ] **Step 1: 创建 timeline.ts**

```typescript
export interface SceneConfig {
  id: number;
  component: string;
  duration: number; // 秒
}

export const scenes: SceneConfig[] = [
  { id: 0, component: "Scene00Cover", duration: 5 },
  { id: 1, component: "Scene01Doubao", duration: 20 },
  { id: 2, component: "Scene02Reality", duration: 15 },
  { id: 3, component: "Scene03Vision", duration: 12 },
  { id: 4, component: "Scene04WhatIs", duration: 13 },
  { id: 5, component: "Scene05Capabilities", duration: 25 },
  { id: 6, component: "Scene06Demo", duration: 25 },
  { id: 7, component: "Scene07CCSwitch", duration: 20 },
  { id: 8, component: "Scene08Roadmap", duration: 10 },
  { id: 9, component: "Scene09CTA", duration: 5 },
];

export const totalDuration = scenes.reduce((sum, s) => sum + s.duration, 0);
// = 150 秒 = 2 分 30 秒
```

- [ ] **Step 2: 创建 App.tsx（骨架，后续追加场景 import）**

```tsx
import { useState, useEffect, useCallback } from "react";
import { scenes } from "./data/timeline";

export default function App() {
  const [currentScene, setCurrentScene] = useState(0);
  const [startTime, setStartTime] = useState(0);

  const advance = useCallback(() => {
    let elapsed = 0;
    const now = Date.now();
    for (let i = 0; i < scenes.length; i++) {
      elapsed += scenes[i].duration * 1000;
      if (now - startTime < elapsed) {
        return i;
      }
    }
    return scenes.length - 1;
  }, [startTime]);

  useEffect(() => {
    setStartTime(Date.now());
  }, []);

  useEffect(() => {
    const timer = setInterval(() => {
      const scene = advance();
      setCurrentScene(scene);
    }, 100); // 每 100ms 检查一次
    return () => clearInterval(timer);
  }, [advance]);

  const isActive = (id: number) => currentScene === id;

  return (
    <div
      className="relative overflow-hidden"
      style={{ width: 1920, height: 1080, background: "#FFFAF5" }}
    >
      {/* 场景占位，后续 Task 逐个替换 */}
      <div style={{ display: isActive(0) ? "flex" : "none" }}>
        Scene 0: Cover (5s)
      </div>
      <div style={{ display: isActive(1) ? "flex" : "none" }}>
        Scene 1: Doubao (20s)
      </div>
      <div style={{ display: isActive(2) ? "flex" : "none" }}>
        Scene 2: Reality (15s)
      </div>
      <div style={{ display: isActive(3) ? "flex" : "none" }}>
        Scene 3: Vision (12s)
      </div>
      <div style={{ display: isActive(4) ? "flex" : "none" }}>
        Scene 4: WhatIs (13s)
      </div>
      <div style={{ display: isActive(5) ? "flex" : "none" }}>
        Scene 5: Capabilities (25s)
      </div>
      <div style={{ display: isActive(6) ? "flex" : "none" }}>
        Scene 6: Demo (25s)
      </div>
      <div style={{ display: isActive(7) ? "flex" : "none" }}>
        Scene 7: CCSwitch (20s)
      </div>
      <div style={{ display: isActive(8) ? "flex" : "none" }}>
        Scene 8: Roadmap (10s)
      </div>
      <div style={{ display: isActive(9) ? "flex" : "none" }}>
        Scene 9: CTA (5s)
      </div>
    </div>
  );
}
```

- [ ] **Step 3: 验证 dev server 能启动**

```bash
cd .claude/promo-video && npm run dev
# 打开浏览器确认 1920×1080 画布渲染、场景自动切换
```

- [ ] **Step 4: Commit**

```bash
git add .claude/promo-video/src/data/timeline.ts .claude/promo-video/src/App.tsx
git commit -m "feat: add scene timeline config and App orchestrator"
```

---

### Task 4: 场景 0 — 封面片头

**Files:**
- Create: `/.claude/promo-video/src/scenes/Scene00Cover.tsx`
- Modify: `/.claude/promo-video/src/App.tsx`（替换占位 div）

- [ ] **Step 1: 创建 Scene00Cover.tsx**

```tsx
import { motion } from "framer-motion";
import { ChevronDown, Box } from "lucide-react";
import SceneWrapper from "../components/SceneWrapper";
import Blob from "../components/Blob";

export default function Scene00Cover({ show }: { show: boolean }) {
  return (
    <SceneWrapper show={show}>
      {/* 渐变背景 */}
      <div
        className="absolute inset-0"
        style={{
          background: "linear-gradient(135deg, #FF6B35 0%, #FFB563 50%, #FFFAF5 100%)",
        }}
      />
      <Blob color="#FFB563" size={700} opacity={0.35} top="45%" left="50%" />
      <Blob color="#FF6B35" size={400} opacity={0.2} top="55%" left="35%" duration={20} />

      {/* Logo */}
      <motion.div
        initial={{ opacity: 0, scale: 0.9, filter: "blur(8px)" }}
        animate={{ opacity: 1, scale: 1, filter: "blur(0px)" }}
        transition={{ duration: 1.2, ease: "easeOut" }}
        className="relative z-10 mb-10"
      >
        <div
          className="rounded-3xl flex items-center justify-center"
          style={{
            width: 140,
            height: 140,
            background: "linear-gradient(135deg, #FF6B35, #FFB563)",
            boxShadow: "0 20px 60px rgba(255,107,53,0.4)",
          }}
        >
          <Box size={64} color="white" />
        </div>
      </motion.div>

      {/* 标题 */}
      <motion.h1
        initial={{ opacity: 0, y: 30 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.8, duration: 0.8 }}
        className="relative z-10 text-white text-8xl font-bold tracking-tight"
        style={{ textShadow: "0 4px 20px rgba(0,0,0,0.15)" }}
      >
        AgenticBoot
      </motion.h1>

      {/* 副标题 */}
      <motion.p
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 1.4, duration: 0.6 }}
        className="relative z-10 text-white/90 text-3xl mt-4 font-medium"
      >
        AI 工具，一点就装
      </motion.p>

      {/* 向下箭头 */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1, y: [0, 8, 0] }}
        transition={{
          opacity: { delay: 2.5, duration: 0.5 },
          y: { delay: 2.5, duration: 1.5, repeat: Infinity },
        }}
        className="absolute bottom-16 z-10"
      >
        <ChevronDown size={36} className="text-white/70" />
      </motion.div>
    </SceneWrapper>
  );
}
```

- [ ] **Step 2: 更新 App.tsx — 替换 Scene 0 占位**

将 App.tsx 中的：
```tsx
<div style={{ display: isActive(0) ? "flex" : "none" }}>
  Scene 0: Cover (5s)
</div>
```
替换为：
```tsx
import Scene00Cover from "./scenes/Scene00Cover";
// ...
<Scene00Cover show={isActive(0)} />
```

- [ ] **Step 3: 预览验证**

```bash
cd .claude/promo-video && npm run dev
# 确认封面动画：blob 呼吸、Logo 模糊到清晰、文字浮入、箭头弹跳
```

- [ ] **Step 4: Commit**

```bash
git add .claude/promo-video/src/scenes/Scene00Cover.tsx .claude/promo-video/src/App.tsx
git commit -m "feat: add Scene 0 — cover with logo reveal and blob animation"
```

---

### Task 5: 共享组件 — BarChart + AnimatedCounter

**Files:**
- Create: `/.claude/promo-video/src/components/BarChart.tsx`
- Create: `/.claude/promo-video/src/components/AnimatedCounter.tsx`

- [ ] **Step 1: 创建 AnimatedCounter.tsx**

```tsx
import { motion, useSpring, useTransform } from "framer-motion";
import { useEffect } from "react";

interface AnimatedCounterProps {
  from?: number;
  to: number;
  suffix?: string;
  duration?: number;
  className?: string;
}

export default function AnimatedCounter({
  from = 0,
  to,
  suffix = "",
  duration = 1.5,
  className = "",
}: AnimatedCounterProps) {
  const spring = useSpring(from, { stiffness: 80, damping: 30 });
  const display = useTransform(spring, (v) => Math.round(v).toString());

  useEffect(() => {
    spring.set(to);
  }, [spring, to]);

  return (
    <span className={className}>
      <motion.span>{display}</motion.span>
      {suffix}
    </span>
  );
}
```

- [ ] **Step 2: 创建 BarChart.tsx**

```tsx
import { motion } from "framer-motion";
import AnimatedCounter from "./AnimatedCounter";

interface BarItem {
  label: string;
  leftValue: string;    // 豆包侧
  rightValue: string;   // 桌面AI侧
  leftBar: number;      // 0-100 百分比高度
  rightBar: number;
}

const barData: BarItem[] = [
  { label: "整理 100 张照片", leftValue: "40min", rightValue: "3min", leftBar: 15, rightBar: 92 },
  { label: "处理 50 份文档/表格", leftValue: "60min", rightValue: "5min", leftBar: 12, rightBar: 90 },
];

export default function BarChart() {
  return (
    <div className="flex flex-col gap-12">
      {barData.map((item, i) => (
        <motion.div
          key={item.label}
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 + i * 1.5, duration: 0.6 }}
          className="flex flex-col gap-3"
        >
          <p className="text-center text-xl text-brand-charcoal font-medium">{item.label}</p>
          <div className="flex items-end justify-center gap-24 h-48">
            {/* 豆包侧 */}
            <div className="flex flex-col items-center gap-2">
              <span className="text-lg text-gray-400 font-medium">{item.leftValue}</span>
              <motion.div
                initial={{ height: 0 }}
                animate={{ height: item.leftBar * 1.8 }}
                transition={{ delay: 0.6 + i * 1.5, duration: 0.8, ease: "easeOut" }}
                className="w-32 rounded-t-xl bg-gray-300"
              />
              <span className="text-sm text-gray-400">仅用豆包</span>
            </div>
            {/* 桌面AI侧 */}
            <div className="flex flex-col items-center gap-2">
              <span className="text-lg text-brand-orange font-bold">{item.rightValue}</span>
              <motion.div
                initial={{ height: 0 }}
                animate={{ height: item.rightBar * 1.8 }}
                transition={{ delay: 0.8 + i * 1.5, duration: 0.8, ease: "easeOut" }}
                className="w-32 rounded-t-xl"
                style={{ background: "linear-gradient(180deg, #FF6B35, #FFB563)" }}
              />
              <span className="text-sm text-brand-orange font-medium">搭配桌面AI</span>
            </div>
          </div>
        </motion.div>
      ))}
    </div>
  );
}
```

- [ ] **Step 3: 验证 BarChart 组件**（dev server 临时渲染到 App 预览，然后移除）

- [ ] **Step 4: Commit**

```bash
git add .claude/promo-video/src/components/AnimatedCounter.tsx .claude/promo-video/src/components/BarChart.tsx
git commit -m "feat: add AnimatedCounter and BarChart shared components"
```

---

### Task 6: 场景 1 — 豆包 vs 桌面 AI 工具

**Files:**
- Create: `/.claude/promo-video/src/scenes/Scene01Doubao.tsx`
- Modify: `/.claude/promo-video/src/App.tsx`

- [ ] **Step 1: 创建 Scene01Doubao.tsx**

```tsx
import { motion } from "framer-motion";
import { Check, X } from "lucide-react";
import SceneWrapper from "../components/SceneWrapper";
import BarChart from "../components/BarChart";

const introIcons = [
  { name: "豆包", color: "#4A90D9" },
  { name: "通义千问", color: "#6C5CE7" },
  { name: "Kimi", color: "#00B894" },
];

export default function Scene01Doubao({ show }: { show: boolean }) {
  return (
    <SceneWrapper show={show}>
      <div className="w-full h-full flex flex-col items-center justify-center px-32">
        {/* 拍1: 认可现有AI */}
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.6 }}
          className="flex flex-col items-center gap-8"
        >
          <div className="flex gap-6">
            {introIcons.map((icon, i) => (
              <motion.div
                key={icon.name}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.2 * i, duration: 0.5 }}
                className="flex flex-col items-center gap-2"
              >
                <div
                  className="w-20 h-20 rounded-2xl flex items-center justify-center text-white text-2xl font-bold shadow-lg"
                  style={{ background: icon.color }}
                >
                  {icon.name.charAt(0)}
                </div>
                <span className="text-lg text-brand-charcoal">{icon.name}</span>
              </motion.div>
            ))}
          </div>
          <motion.p
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.8, duration: 0.5 }}
            className="text-3xl text-brand-charcoal font-medium"
          >
            它们让 AI 走进了日常生活
          </motion.p>
        </motion.div>

        {/* 拍2: 数据对比柱状图 */}
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 3.2, duration: 0.6 }}
          className="mt-12"
        >
          <p className="text-2xl text-brand-charcoal mb-8 text-center">
            但有些事，豆包做不到
          </p>
          <BarChart />

          {/* 第三组：做不到的对比 */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 6.5, duration: 0.6 }}
            className="flex flex-col items-center gap-3 mt-10"
          >
            <p className="text-xl text-brand-charcoal font-medium">
              搭建本地自动化工作流
            </p>
            <div className="flex gap-20 items-center">
              <div className="flex flex-col items-center gap-2">
                <X size={40} className="text-red-400" />
                <span className="text-sm text-gray-400">仅用豆包 — 无法触及本地文件</span>
              </div>
              <div className="flex flex-col items-center gap-2">
                <Check size={40} className="text-brand-mint" />
                <span className="text-sm text-brand-mint font-medium">桌面AI — 直接操作</span>
              </div>
            </div>
          </motion.div>
        </motion.div>

        {/* 拍3: 结论 */}
        <motion.div
          initial={{ opacity: 0, scale: 0.9 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ delay: 10, duration: 0.8 }}
          className="absolute bottom-24"
        >
          <h2
            className="text-7xl font-bold"
            style={{
              background: "linear-gradient(135deg, #FF6B35, #FFB563)",
              WebkitBackgroundClip: "text",
              WebkitTextFillColor: "transparent",
            }}
          >
            效率平均提升 90%+
          </h2>
        </motion.div>
      </div>
    </SceneWrapper>
  );
}
```

- [ ] **Step 2: 更新 App.tsx — 替换 Scene 1 占位**

```tsx
import Scene01Doubao from "./scenes/Scene01Doubao";
// 替换占位 div 为:
<Scene01Doubao show={isActive(1)} />
```

- [ ] **Step 3: 预览验证**

```bash
cd .claude/promo-video && npm run dev
# 确认：三阶段时序正确、柱状图动画流畅、结论大字渐变
```

- [ ] **Step 4: Commit**

```bash
git add .claude/promo-video/src/scenes/Scene01Doubao.tsx .claude/promo-video/src/App.tsx
git commit -m "feat: add Scene 1 — Doubao vs Desktop AI data comparison"
```

---

### Task 7: 场景 2 — 现实落差

**Files:**
- Create: `/.claude/promo-video/src/scenes/Scene02Reality.tsx`
- Modify: `/.claude/promo-video/src/App.tsx`

- [ ] **Step 1: 创建 Scene02Reality.tsx**

```tsx
import { motion } from "framer-motion";
import SceneWrapper from "../components/SceneWrapper";

const walls = ["终端命令行", "环境变量", "PATH 配置", "Node.js 版本"];

export default function Scene02Reality({ show }: { show: boolean }) {
  return (
    <SceneWrapper show={show}>
      <div
        className="w-full h-full flex flex-col items-center justify-center relative"
        style={{ background: "linear-gradient(180deg, #e8ecf1 0%, #d5dbe3 100%)" }}
      >
        {/* 拍1: 障碍墙砸下 */}
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.5 }}
          className="text-center"
        >
          <p className="text-3xl text-brand-charcoal mb-12">想用 AI 桌面工具？先过这些关</p>
          <div className="flex gap-6">
            {walls.map((wall, i) => (
              <motion.div
                key={wall}
                initial={{ y: -200, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{
                  delay: 0.5 + i * 0.6,
                  duration: 0.5,
                  type: "spring",
                  stiffness: 200,
                  damping: 15,
                }}
                className="w-44 h-28 bg-white/80 backdrop-blur rounded-xl shadow-lg flex items-center justify-center"
              >
                <p className="text-lg text-gray-500 font-medium">{wall}</p>
              </motion.div>
            ))}
          </div>
        </motion.div>

        {/* 拍2: 人物 + 劝退 */}
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 8, duration: 0.8 }}
          className="absolute inset-0 flex flex-col items-center justify-center"
          style={{ background: "rgba(45,52,54,0.85)" }}
        >
          {/* 人物剪影 */}
          <motion.div
            initial={{ scale: 0.8, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            transition={{ delay: 8.5, duration: 0.6 }}
            className="mb-8"
          >
            <div className="w-32 h-32 rounded-full bg-white/10 flex items-center justify-center">
              <span className="text-6xl">😵</span>
            </div>
          </motion.div>
          <motion.p
            initial={{ y: 20, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
            transition={{ delay: 9.2, duration: 0.6 }}
            className="text-white text-5xl font-bold text-center"
          >
            还没见到 AI
          </motion.p>
          <motion.p
            initial={{ y: 20, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
            transition={{ delay: 9.8, duration: 0.6 }}
            className="text-white/70 text-4xl mt-4"
          >
            先被安装劝退
          </motion.p>
        </motion.div>
      </div>
    </SceneWrapper>
  );
}
```

- [ ] **Step 2: 更新 App.tsx — 替换 Scene 2 占位**

```tsx
import Scene02Reality from "./scenes/Scene02Reality";
// ...
<Scene02Reality show={isActive(2)} />
```

- [ ] **Step 3: 预览验证**

- [ ] **Step 4: Commit**

```bash
git add .claude/promo-video/src/scenes/Scene02Reality.tsx .claude/promo-video/src/App.tsx
git commit -m "feat: add Scene 2 — installation barrier reality check"
```

---

### Task 8: 场景 3 — 愿景

**Files:**
- Create: `/.claude/promo-video/src/scenes/Scene03Vision.tsx`
- Modify: `/.claude/promo-video/src/App.tsx`

- [ ] **Step 1: 创建 Scene03Vision.tsx**

```tsx
import { motion } from "framer-motion";
import SceneWrapper from "../components/SceneWrapper";
import Blob from "../components/Blob";

const phrases = [
  { text: "让所有人", delay: 0.3 },
  { text: "不管懂不懂技术", delay: 2.5 },
  { text: "都能便捷使用 AI 工具", delay: 5.0 },
];

export default function Scene03Vision({ show }: { show: boolean }) {
  return (
    <SceneWrapper show={show}>
      <div
        className="absolute inset-0"
        style={{
          background: "linear-gradient(135deg, #FF6B35 0%, #FFB563 50%, #FFFAF5 100%)",
        }}
      />
      <Blob color="#FFB563" size={800} opacity={0.3} top="50%" left="50%" />

      <div className="relative z-10 flex flex-col items-center gap-8">
        {phrases.map((phrase, i) => (
          <motion.p
            key={phrase.text}
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: phrase.delay, duration: 0.8 }}
            className={`text-white text-center ${
              i === 2 ? "text-6xl font-bold" : "text-5xl font-medium"
            }`}
            style={{ textShadow: "0 4px 20px rgba(0,0,0,0.12)" }}
          >
            {phrase.text}
          </motion.p>
        ))}

        {/* 底部拉平线 */}
        <motion.div
          initial={{ scaleX: 0 }}
          animate={{ scaleX: 1 }}
          transition={{ delay: 7, duration: 1.5, ease: "easeInOut" }}
          className="w-96 h-0.5 bg-white/60 mt-8"
          style={{ transformOrigin: "left" }}
        />
      </div>
    </SceneWrapper>
  );
}
```

- [ ] **Step 2: 更新 App.tsx**

```tsx
import Scene03Vision from "./scenes/Scene03Vision";
// ...
<Scene03Vision show={isActive(3)} />
```

- [ ] **Step 3: 预览验证**

- [ ] **Step 4: Commit**

```bash
git add .claude/promo-video/src/scenes/Scene03Vision.tsx .claude/promo-video/src/App.tsx
git commit -m "feat: add Scene 3 — vision statement reveal"
```

---

### Task 9: 共享组件 — FlowArrow + 场景 4

**Files:**
- Create: `/.claude/promo-video/src/components/FlowArrow.tsx`
- Create: `/.claude/promo-video/src/scenes/Scene04WhatIs.tsx`
- Modify: `/.claude/promo-video/src/App.tsx`

- [ ] **Step 1: 创建 FlowArrow.tsx**

```tsx
import { motion } from "framer-motion";

interface FlowArrowProps {
  from?: { x: number; y: number };
  to?: { x: number; y: number };
  width?: number;
}

export default function FlowArrow({ width = 100 }: FlowArrowProps) {
  return (
    <div className="flex items-center" style={{ width }}>
      <motion.div
        className="h-0.5 bg-brand-orange flex-1"
        initial={{ scaleX: 0 }}
        animate={{ scaleX: 1 }}
        transition={{ duration: 0.8, ease: "easeInOut" }}
        style={{ transformOrigin: "left" }}
      />
      <motion.div
        initial={{ opacity: 0, x: -5 }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ delay: 0.6, duration: 0.4 }}
        className="w-0 h-0"
        style={{
          borderTop: "6px solid transparent",
          borderBottom: "6px solid transparent",
          borderLeft: "10px solid #FF6B35",
        }}
      />
    </div>
  );
}
```

- [ ] **Step 2: 创建 Scene04WhatIs.tsx**

```tsx
import { motion } from "framer-motion";
import { Search, Package, LayoutDashboard } from "lucide-react";
import SceneWrapper from "../components/SceneWrapper";
import FlowArrow from "../components/FlowArrow";

const steps = [
  { icon: Search, label: "检测已有环境", color: "#FF6B35" },
  { icon: Package, label: "补齐缺失工具", color: "#FFB563" },
  { icon: LayoutDashboard, label: "统一管理", color: "#26C9A5" },
];

export default function Scene04WhatIs({ show }: { show: boolean }) {
  return (
    <SceneWrapper show={show}>
      <div className="w-full h-full flex flex-col items-center justify-center gap-12">
        <motion.h2
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.5 }}
          className="text-5xl font-bold text-brand-charcoal"
        >
          AgenticBoot 是什么？
        </motion.h2>

        {/* 流程图 */}
        <div className="flex items-center gap-6">
          {steps.map((step, i) => (
            <div key={step.label} className="flex items-center gap-6">
              <motion.div
                initial={{ opacity: 0, y: 30 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.5 + i * 1.2, duration: 0.6 }}
                className="flex flex-col items-center gap-4"
              >
                <motion.div
                  animate={{ scale: [1, 1.1, 1] }}
                  transition={{ delay: 1 + i * 1.2, duration: 0.6 }}
                  className="w-32 h-32 rounded-3xl flex items-center justify-center shadow-xl"
                  style={{ background: step.color }}
                >
                  <step.icon size={48} color="white" />
                </motion.div>
                <span className="text-xl font-medium text-brand-charcoal">{step.label}</span>
              </motion.div>
              {i < steps.length - 1 && (
                <motion.div
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  transition={{ delay: 0.8 + i * 1.2 }}
                >
                  <FlowArrow width={80} />
                </motion.div>
              )}
            </div>
          ))}
        </div>

        <motion.p
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 4.5, duration: 0.6 }}
          className="text-3xl text-brand-charcoal/70 font-medium"
        >
          一个入口，搞定 AI 工具的安装与管理
        </motion.p>
      </div>
    </SceneWrapper>
  );
}
```

- [ ] **Step 3: 更新 App.tsx**

```tsx
import Scene04WhatIs from "./scenes/Scene04WhatIs";
// ...
<Scene04WhatIs show={isActive(4)} />
```

- [ ] **Step 4: 预览验证 + Commit**

```bash
git add .claude/promo-video/src/components/FlowArrow.tsx .claude/promo-video/src/scenes/Scene04WhatIs.tsx .claude/promo-video/src/App.tsx
git commit -m "feat: add FlowArrow component and Scene 4 — what is AgenticBoot"
```

---

### Task 10: 共享组件 — Card + 场景 5（四大核心能力）

**Files:**
- Create: `/.claude/promo-video/src/components/Card.tsx`
- Create: `/.claude/promo-video/src/scenes/Scene05Capabilities.tsx`
- Modify: `/.claude/promo-video/src/App.tsx`

- [ ] **Step 1: 创建 Card.tsx**

```tsx
import { motion } from "framer-motion";
import { LucideIcon } from "lucide-react";

interface CardProps {
  icon: LucideIcon;
  title: string;
  description: string;
  color: string;
  animation: React.ReactNode;
}

export default function Card({ icon: Icon, title, description, color, animation }: CardProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 40 }}
      animate={{ opacity: 1, y: 0 }}
      className="flex items-center gap-8 bg-white rounded-3xl px-12 py-10 shadow-xl"
      style={{ width: 800, boxShadow: "0 10px 40px rgba(0,0,0,0.06)" }}
    >
      <div className="relative w-32 h-32 flex-shrink-0">
        <div
          className="w-full h-full rounded-2xl flex items-center justify-center"
          style={{ background: `${color}15` }}
        >
          <Icon size={48} color={color} />
        </div>
        <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
          {animation}
        </div>
      </div>
      <div className="flex flex-col gap-2">
        <h3 className="text-3xl font-bold text-brand-charcoal">{title}</h3>
        <p className="text-xl text-brand-charcoal/60">{description}</p>
      </div>
    </motion.div>
  );
}
```

- [ ] **Step 2: 创建 Scene05Capabilities.tsx**

```tsx
import { motion } from "framer-motion";
import { Search, Download, LayoutDashboard, Eye } from "lucide-react";
import SceneWrapper from "../components/SceneWrapper";
import Card from "../components/Card";

const capabilities = [
  {
    icon: Search,
    title: "智能检测",
    description: "先检测已装工具，避免重复安装。已能用的直接复用。",
    color: "#FF6B35",
    animation: (
      <motion.div
        initial={{ top: "10%" }}
        animate={{ top: ["10%", "70%", "10%"] }}
        transition={{ duration: 2.5, repeat: Infinity, ease: "easeInOut" }}
        className="absolute left-2 right-2 h-0.5 bg-brand-orange/40"
      />
    ),
  },
  {
    icon: Download,
    title: "一键安装",
    description: "点一下，剩下的交给我们。进度、状态全程可见。",
    color: "#FFB563",
    animation: (
      <motion.div
        initial={{ width: "0%" }}
        animate={{ width: "100%" }}
        transition={{ delay: 0.5, duration: 2, ease: "easeInOut" }}
        className="absolute bottom-3 left-3 right-3 h-2 bg-brand-gold rounded-full"
      />
    ),
  },
  {
    icon: LayoutDashboard,
    title: "统一管理",
    description: "受管安装和系统已有安装，一个界面全看到。",
    color: "#26C9A5",
    animation: (
      <motion.div className="flex gap-1">
        {[0, 1, 2].map((i) => (
          <motion.div
            key={i}
            initial={{ y: 20, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
            transition={{ delay: i * 0.3, duration: 0.4 }}
            className="w-3 h-3 rounded-sm bg-brand-mint"
          />
        ))}
      </motion.div>
    ),
  },
  {
    icon: Eye,
    title: "过程透明",
    description: "安装过程不黑箱。实时日志，问题一眼定位。",
    color: "#6C5CE7",
    animation: (
      <div className="flex flex-col gap-1">
        {[0, 1, 2].map((i) => (
          <motion.div
            key={i}
            initial={{ width: 0 }}
            animate={{ width: [0, i === 0 ? 60 : i === 1 ? 80 : 50] }}
            transition={{ delay: i * 0.5, duration: 0.6 }}
            className="h-1.5 rounded-full bg-purple-300"
          />
        ))}
      </div>
    ),
  },
];

export default function Scene05Capabilities({ show }: { show: boolean }) {
  return (
    <SceneWrapper show={show}>
      <div className="w-full h-full flex flex-col items-center justify-center gap-8">
        <motion.h2
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className="text-5xl font-bold text-brand-charcoal mb-4"
        >
          四大核心能力
        </motion.h2>
        <div className="flex flex-col gap-6">
          {capabilities.map((cap, i) => (
            <motion.div
              key={cap.title}
              initial={{ opacity: 0, x: -60 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: 0.5 + i * 2.5, duration: 0.5 }}
            >
              <Card {...cap} />
            </motion.div>
          ))}
        </div>
      </div>
    </SceneWrapper>
  );
}
```

- [ ] **Step 3: 更新 App.tsx + 预览 + Commit**

```bash
git add .claude/promo-video/src/components/Card.tsx .claude/promo-video/src/scenes/Scene05Capabilities.tsx .claude/promo-video/src/App.tsx
git commit -m "feat: add Card component and Scene 5 — four core capabilities"
```

---

### Task 11: 场景 6 — 操作演示

**Files:**
- Create: `/.claude/promo-video/src/scenes/Scene06Demo.tsx`
- Modify: `/.claude/promo-video/src/App.tsx`

- [ ] **Step 1: 创建 Scene06Demo.tsx**

```tsx
import { motion } from "framer-motion";
import { Check, ChevronRight } from "lucide-react";
import SceneWrapper from "../components/SceneWrapper";

const wizardSteps = [
  { label: "检测环境", done: true },
  { label: "选择工具", done: true },
  { label: "一键安装", done: false },
];

const managerTools = [
  { name: "Claude Code", status: "已安装", color: "#26C9A5" },
  { name: "Codex CLI", status: "已安装", color: "#26C9A5" },
  { name: "Gemini CLI", status: "已安装", color: "#26C9A5" },
  { name: "OpenCode", status: "可安装", color: "#FFB563" },
  { name: "Hermes", status: "已安装", color: "#26C9A5" },
];

export default function Scene06Demo({ show }: { show: boolean }) {
  return (
    <SceneWrapper show={show}>
      <div className="w-full h-full flex items-center justify-center gap-16 px-20">
        {/* 左侧: Wizard mockup */}
        <motion.div
          initial={{ opacity: 0, x: -40 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ duration: 0.6 }}
          className="w-[700px] bg-white rounded-3xl shadow-2xl overflow-hidden"
          style={{ boxShadow: "0 20px 80px rgba(0,0,0,0.1)" }}
        >
          {/* 窗口标题栏 */}
          <div className="h-12 bg-gray-50 flex items-center px-5 gap-2 border-b border-gray-100">
            <div className="w-3 h-3 rounded-full bg-red-400" />
            <div className="w-3 h-3 rounded-full bg-yellow-400" />
            <div className="w-3 h-3 rounded-full bg-green-400" />
            <span className="ml-4 text-sm text-gray-400">装机向导</span>
          </div>
          {/* Wizard 内容 */}
          <div className="p-10">
            <h3 className="text-2xl font-bold text-brand-charcoal mb-8">欢迎使用 AgenticBoot</h3>
            <div className="flex flex-col gap-4">
              {wizardSteps.map((step, i) => (
                <motion.div
                  key={step.label}
                  initial={{ opacity: 0, x: -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ delay: 0.3 + i * 0.5 }}
                  className="flex items-center gap-4"
                >
                  <motion.div
                    animate={!step.done ? { scale: [1, 1.15, 1] } : {}}
                    transition={{ repeat: Infinity, duration: 1.5 }}
                    className={`w-10 h-10 rounded-full flex items-center justify-center ${
                      step.done ? "bg-brand-mint" : "bg-brand-orange"
                    }`}
                  >
                    {step.done ? (
                      <Check size={20} color="white" />
                    ) : (
                      <ChevronRight size={20} color="white" />
                    )}
                  </motion.div>
                  <span className={`text-xl ${step.done ? "text-gray-400" : "text-brand-charcoal font-medium"}`}>
                    {step.label}
                  </span>
                </motion.div>
              ))}
            </div>
            {/* 进度条 */}
            <motion.div
              initial={{ width: "0%" }}
              animate={{ width: "66%" }}
              transition={{ delay: 1.5, duration: 2 }}
              className="h-2 rounded-full mt-8"
              style={{ background: "linear-gradient(90deg, #FF6B35, #FFB563)" }}
            />
          </div>
        </motion.div>

        {/* 右侧: Manager mockup */}
        <motion.div
          initial={{ opacity: 0, x: 40 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.3, duration: 0.6 }}
          className="w-[700px] bg-white rounded-3xl shadow-2xl overflow-hidden"
          style={{ boxShadow: "0 20px 80px rgba(0,0,0,0.1)" }}
        >
          {/* 窗口标题栏 */}
          <div className="h-12 bg-gray-50 flex items-center px-5 gap-2 border-b border-gray-100">
            <div className="w-3 h-3 rounded-full bg-red-400" />
            <div className="w-3 h-3 rounded-full bg-yellow-400" />
            <div className="w-3 h-3 rounded-full bg-green-400" />
            <span className="ml-4 text-sm text-gray-400">软件管家</span>
          </div>
          {/* Manager 内容 */}
          <div className="p-10">
            <h3 className="text-2xl font-bold text-brand-charcoal mb-6">已安装 / 可安装</h3>
            <div className="flex flex-col gap-3">
              {managerTools.map((tool, i) => (
                <motion.div
                  key={tool.name}
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: 1 + i * 0.3 }}
                  className="flex items-center justify-between px-5 py-3 rounded-xl bg-gray-50 hover:bg-gray-100 transition-colors"
                >
                  <span className="text-lg font-medium text-brand-charcoal">{tool.name}</span>
                  <span
                    className="text-sm font-medium px-3 py-1 rounded-full"
                    style={{ background: `${tool.color}20`, color: tool.color }}
                  >
                    {tool.status}
                  </span>
                </motion.div>
              ))}
            </div>
          </div>
        </motion.div>
      </div>
    </SceneWrapper>
  );
}
```

- [ ] **Step 2: 更新 App.tsx + 预览 + Commit**

```bash
git add .claude/promo-video/src/scenes/Scene06Demo.tsx .claude/promo-video/src/App.tsx
git commit -m "feat: add Scene 6 — Wizard and Manager UI mockup demo"
```

---

### Task 12: 场景 7 — 为什么基于 CC Switch

**Files:**
- Create: `/.claude/promo-video/src/scenes/Scene07CCSwitch.tsx`
- Modify: `/.claude/promo-video/src/App.tsx`

- [ ] **Step 1: 创建 Scene07CCSwitch.tsx**

```tsx
import { motion } from "framer-motion";
import { GitFork, Lightbulb } from "lucide-react";
import SceneWrapper from "../components/SceneWrapper";
import FlowArrow from "../components/FlowArrow";

export default function Scene07CCSwitch({ show }: { show: boolean }) {
  return (
    <SceneWrapper show={show}>
      <div className="w-full h-full flex flex-col items-center justify-center gap-16">
        <div className="flex items-center gap-12">
          {/* 左侧: CC Switch */}
          <motion.div
            initial={{ opacity: 0, x: -60 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.6 }}
            className="flex flex-col items-center gap-6"
          >
            <div className="w-40 h-40 rounded-3xl bg-gray-100 flex items-center justify-center shadow-lg">
              <GitFork size={64} className="text-gray-400" />
            </div>
            <h3 className="text-3xl font-bold text-gray-500">CC Switch</h3>
            <ul className="text-lg text-gray-400 space-y-2 text-center">
              <li>成熟桌面框架</li>
              <li>跨平台基础能力</li>
              <li>工具生态积累</li>
            </ul>
          </motion.div>

          {/* 中间箭头 */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.8, duration: 0.6 }}
            className="flex flex-col items-center gap-2"
          >
            <FlowArrow width={120} />
            <span className="text-sm text-brand-charcoal/40">增量二开</span>
          </motion.div>

          {/* 右侧: AgenticBoot */}
          <motion.div
            initial={{ opacity: 0, x: 60 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.4, duration: 0.6 }}
            className="flex flex-col items-center gap-6"
          >
            <div
              className="w-40 h-40 rounded-3xl flex items-center justify-center shadow-xl"
              style={{
                background: "linear-gradient(135deg, #FF6B35, #FFB563)",
                boxShadow: "0 20px 60px rgba(255,107,53,0.3)",
              }}
            >
              <Lightbulb size={64} color="white" />
            </div>
            <h3 className="text-3xl font-bold text-brand-orange">AgenticBoot</h3>
            <ul className="text-lg text-brand-charcoal space-y-2 text-center">
              <li>智能装机检测</li>
              <li>统一工具管理</li>
              <li>体验深度创新</li>
            </ul>
          </motion.div>
        </div>

        {/* 底部结论 */}
        <motion.p
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 2, duration: 0.6 }}
          className="text-4xl font-bold text-brand-charcoal"
        >
          站在成熟的肩膀上
        </motion.p>
        <motion.p
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 3, duration: 0.6 }}
          className="text-2xl text-brand-charcoal/60 -mt-8"
        >
          把精力全部投入装机体验创新
        </motion.p>
      </div>
    </SceneWrapper>
  );
}
```

- [ ] **Step 2: 更新 App.tsx + 预览 + Commit**

```bash
git add .claude/promo-video/src/scenes/Scene07CCSwitch.tsx .claude/promo-video/src/App.tsx
git commit -m "feat: add Scene 7 — why based on CC Switch"
```

---

### Task 13: 共享组件 — Timeline + 场景 8（未来路线图）

**Files:**
- Create: `/.claude/promo-video/src/components/Timeline.tsx`
- Create: `/.claude/promo-video/src/scenes/Scene08Roadmap.tsx`
- Modify: `/.claude/promo-video/src/App.tsx`

- [ ] **Step 1: 创建 Timeline.tsx**

```tsx
import { motion } from "framer-motion";
import { Check } from "lucide-react";

interface TimelineNode {
  label: string;
  status: "done" | "in_progress" | "planned";
}

interface TimelineProps {
  nodes: TimelineNode[];
}

const statusConfig = {
  done: { dotClass: "bg-brand-mint", lineClass: "bg-brand-mint", icon: true },
  in_progress: { dotClass: "bg-brand-orange", lineClass: "bg-brand-orange/30", icon: false },
  planned: { dotClass: "border-2 border-dashed border-gray-300 bg-transparent", lineClass: "bg-gray-200", icon: false },
};

export default function Timeline({ nodes }: TimelineProps) {
  return (
    <div className="flex items-start gap-0 relative">
      {nodes.map((node, i) => {
        const config = statusConfig[node.status];
        return (
          <div key={node.label} className="flex flex-col items-center" style={{ width: 300 }}>
            {/* 横线 + 节点 */}
            <div className="flex items-center w-full">
              {i > 0 && (
                <motion.div
                  initial={{ scaleX: 0 }}
                  animate={{ scaleX: 1 }}
                  transition={{ delay: 0.5 + i * 0.8, duration: 0.6 }}
                  className={`h-0.5 flex-1 ${config.lineClass}`}
                  style={{ transformOrigin: "left" }}
                />
              )}
              <motion.div
                initial={{ scale: 0 }}
                animate={{ scale: 1 }}
                transition={{ delay: 0.3 + i * 0.8, type: "spring", stiffness: 200 }}
                className={`w-8 h-8 rounded-full flex items-center justify-center ${config.dotClass} ${
                  node.status === "in_progress" ? "animate-pulse" : ""
                }`}
              >
                {config.icon && <Check size={14} color="white" />}
              </motion.div>
              {i < nodes.length - 1 && (
                <motion.div
                  initial={{ scaleX: 0 }}
                  animate={{ scaleX: 1 }}
                  transition={{ delay: 0.5 + i * 0.8, duration: 0.6 }}
                  className={`h-0.5 flex-1 ${config.lineClass}`}
                  style={{ transformOrigin: "left" }}
                />
              )}
            </div>
            {/* 标签 */}
            <motion.p
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.7 + i * 0.8 }}
              className="text-lg font-medium text-brand-charcoal mt-3 text-center"
            >
              {node.label}
            </motion.p>
          </div>
        );
      })}
    </div>
  );
}
```

- [ ] **Step 2: 创建 Scene08Roadmap.tsx**

```tsx
import { motion } from "framer-motion";
import SceneWrapper from "../components/SceneWrapper";
import Timeline from "../components/Timeline";

const nodes = [
  { label: "Windows 装机链路 ✅", status: "done" as const },
  { label: "更多 AI 工具 + 体验打磨", status: "in_progress" as const },
  { label: "macOS / Linux 支持", status: "planned" as const },
];

export default function Scene08Roadmap({ show }: { show: boolean }) {
  return (
    <SceneWrapper show={show}>
      <div className="w-full h-full flex flex-col items-center justify-center gap-12">
        <motion.h2
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className="text-5xl font-bold text-brand-charcoal"
        >
          未来路线图
        </motion.h2>
        <Timeline nodes={nodes} />
        <motion.p
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 3, duration: 0.6 }}
          className="text-2xl text-brand-charcoal/50 mt-8"
        >
          这只是开始
        </motion.p>
      </div>
    </SceneWrapper>
  );
}
```

- [ ] **Step 3: 更新 App.tsx + 预览 + Commit**

```bash
git add .claude/promo-video/src/components/Timeline.tsx .claude/promo-video/src/scenes/Scene08Roadmap.tsx .claude/promo-video/src/App.tsx
git commit -m "feat: add Timeline component and Scene 8 — future roadmap"
```

---

### Task 14: 场景 9 — 结尾 CTA

**Files:**
- Create: `/.claude/promo-video/src/scenes/Scene09CTA.tsx`
- Modify: `/.claude/promo-video/src/App.tsx`

- [ ] **Step 1: 创建 Scene09CTA.tsx**

```tsx
import { motion } from "framer-motion";
import { Box, Star } from "lucide-react";
import SceneWrapper from "../components/SceneWrapper";
import Blob from "../components/Blob";

export default function Scene09CTA({ show }: { show: boolean }) {
  return (
    <SceneWrapper show={show}>
      <div
        className="absolute inset-0"
        style={{
          background: "linear-gradient(135deg, #FF6B35 0%, #FFB563 50%, #FFFAF5 100%)",
        }}
      />
      <Blob color="#FFB563" size={700} opacity={0.3} top="50%" left="50%" />

      <div className="relative z-10 flex flex-col items-center gap-8">
        {/* Logo */}
        <motion.div
          initial={{ opacity: 0, scale: 0.8 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 0.6 }}
        >
          <div
            className="rounded-3xl flex items-center justify-center"
            style={{
              width: 100,
              height: 100,
              background: "rgba(255,255,255,0.25)",
              backdropFilter: "blur(10px)",
            }}
          >
            <Box size={48} color="white" />
          </div>
        </motion.div>

        <motion.h2
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.4, duration: 0.6 }}
          className="text-6xl font-bold text-white"
          style={{ textShadow: "0 4px 20px rgba(0,0,0,0.12)" }}
        >
          让 AI 工具触手可及
        </motion.h2>

        {/* GitHub info */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 1.0, duration: 0.6 }}
          className="flex flex-col items-center gap-4 mt-4"
        >
          <p className="text-2xl text-white/80 font-mono">
            github.com/unbound9527/agenticboot
          </p>
          <div className="flex items-center gap-3 px-6 py-3 bg-white/20 backdrop-blur rounded-full">
            <Star size={24} color="white" fill="white" />
            <span className="text-xl text-white font-medium">Star on GitHub</span>
          </div>
        </motion.div>

        {/* 二维码占位 */}
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 1.6, duration: 0.6 }}
          className="w-32 h-32 bg-white/20 backdrop-blur rounded-2xl flex items-center justify-center mt-4"
        >
          <span className="text-white/60 text-sm">扫码访问</span>
        </motion.div>
      </div>
    </SceneWrapper>
  );
}
```

- [ ] **Step 2: 更新 App.tsx + 预览 + Commit**

```bash
git add .claude/promo-video/src/scenes/Scene09CTA.tsx .claude/promo-video/src/App.tsx
git commit -m "feat: add Scene 9 — ending CTA"
```

---

### Task 15: Playwright 录制脚本

**Files:**
- Create: `/.claude/promo-video/scripts/record.ts`

- [ ] **Step 1: 创建 scripts/record.ts**

```typescript
import { chromium } from "playwright";
import * as fs from "fs";
import * as path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUTPUT_DIR = path.resolve(__dirname, "..", "output");
const FRAMES_DIR = path.join(OUTPUT_DIR, "frames");
const WIDTH = 1920;
const HEIGHT = 1080;
const FPS = 30;
const DURATION_SECONDS = 150; // 2分30秒
const TOTAL_FRAMES = FPS * DURATION_SECONDS;

async function main() {
  // 清空并创建帧目录
  if (fs.existsSync(FRAMES_DIR)) {
    fs.rmSync(FRAMES_DIR, { recursive: true });
  }
  fs.mkdirSync(FRAMES_DIR, { recursive: true });

  // 启动浏览器
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  await page.setViewportSize({ width: WIDTH, height: HEIGHT });

  // 打开页面（假设 Vite dev server 在 localhost:5173）
  await page.goto("http://localhost:5173", { waitUntil: "networkidle" });

  // 等待 React 渲染
  await page.waitForSelector("#root > div");

  console.log(`开始录制: ${TOTAL_FRAMES} 帧, ${DURATION_SECONDS}s @ ${FPS}fps`);

  const frameInterval = 1000 / FPS; // ~33.3ms
  let lastLogPercent = -1;

  for (let i = 0; i < TOTAL_FRAMES; i++) {
    const startTime = performance.now();

    const filePath = path.join(FRAMES_DIR, `frame-${String(i + 1).padStart(5, "0")}.png`);
    await page.screenshot({ path: filePath, type: "png" });

    const percent = Math.round((i / TOTAL_FRAMES) * 100);
    if (percent !== lastLogPercent && percent % 10 === 0) {
      console.log(`  进度: ${percent}% (${i + 1}/${TOTAL_FRAMES})`);
      lastLogPercent = percent;
    }

    // 控制帧率
    const elapsed = performance.now() - startTime;
    const waitTime = Math.max(0, frameInterval - elapsed);
    if (waitTime > 0) {
      await new Promise((r) => setTimeout(r, waitTime));
    }
  }

  await browser.close();
  console.log(`录制完成！帧文件: ${FRAMES_DIR}`);
  console.log(`运行合成: bash scripts/compose.sh`);
}

main().catch((err) => {
  console.error("录制失败:", err);
  process.exit(1);
});
```

- [ ] **Step 2: Commit**

```bash
git add .claude/promo-video/scripts/record.ts
git commit -m "feat: add Playwright frame recording script"
```

---

### Task 16: ffmpeg 合成脚本

**Files:**
- Create: `/.claude/promo-video/scripts/compose.sh`

- [ ] **Step 1: 创建 scripts/compose.sh**

```bash
#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$PROJECT_DIR/output"
FRAMES_DIR="$OUTPUT_DIR/frames"
OUTPUT_FILE="$OUTPUT_DIR/agenticboot-promo.mp4"

if [ ! -d "$FRAMES_DIR" ]; then
  echo "错误: 帧目录不存在: $FRAMES_DIR"
  echo "请先运行: npm run record"
  exit 1
fi

FRAME_COUNT=$(ls "$FRAMES_DIR"/*.png 2>/dev/null | wc -l)
echo "找到 $FRAME_COUNT 帧, 开始合成..."

ffmpeg -y \
  -framerate 30 \
  -i "$FRAMES_DIR/frame-%05d.png" \
  -c:v libx264 \
  -preset slow \
  -crf 18 \
  -pix_fmt yuv420p \
  -movflags +faststart \
  "$OUTPUT_FILE"

echo "合成完成: $OUTPUT_FILE"
```

- [ ] **Step 2: 添加执行权限**

```bash
chmod +x .claude/promo-video/scripts/compose.sh
```

- [ ] **Step 3: Commit**

```bash
git add .claude/promo-video/scripts/compose.sh
git commit -m "feat: add ffmpeg compose script"
```

---

### Task 17: 全流程验证 + 调优

- [ ] **Step 1: 完整预览**

```bash
cd .claude/promo-video && npm run dev
# 在浏览器中完整预览全部 10 个场景, 检查:
# - 场景切换时序正确
# - 动画流畅无卡顿
# - 文字大小在 1920×1080 分辨率下可读
# - 颜色一致、无闪烁
```

- [ ] **Step 2: 调优问题清单**

  1. 场景 1 的三阶段延迟是否与 20s 分配匹配
  2. 场景 5 四张卡片是否在 25s 内全部露出且不拥挤
  3. 场景 6 的 mockup 窗口大小是否在 1920×1080 下比例恰当
  4. 场景间过渡的 AnimatePresence 是否流畅

- [ ] **Step 3: 构建验证**

```bash
cd .claude/promo-video && npm run build
# 确认无 TS 错误，build 成功
```

- [ ] **Step 4: 调优后 Commit**

```bash
git add .claude/promo-video/src/
git commit -m "fix: tune scene timings and animation parameters"
```

---

## 附录：场景延迟时间表

App.tsx 通过 `Date.now() - startTime` 驱动场景切换。各场景的 `show` 属性在对应时间窗口内为 `true`：

| 时间 | 场景 | 事件 |
|------|------|------|
| 0:00 – 0:05 | Scene 0 | 封面 |
| 0:05 – 0:25 | Scene 1 | 豆包对比 |
| 0:25 – 0:40 | Scene 2 | 现实落差 |
| 0:40 – 0:52 | Scene 3 | 愿景 |
| 0:52 – 1:05 | Scene 4 | 是什么 |
| 1:05 – 1:30 | Scene 5 | 核心能力 |
| 1:30 – 1:55 | Scene 6 | 操作演示 |
| 1:55 – 2:15 | Scene 7 | CC Switch |
| 2:15 – 2:25 | Scene 8 | 路线图 |
| 2:25 – 2:30 | Scene 9 | CTA |
