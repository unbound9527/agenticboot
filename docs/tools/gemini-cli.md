# Gemini CLI 安装教程

## 前置要求

**必须先安装 Node.js（18.0.0 或更高版本）**。

验证：

```bash
node --version
npm --version
```

未安装先看 [Node.js 安装教程](./nodejs.md)。

---

## Windows / macOS / Linux（通用）

### 安装

```bash
npm install -g @google/gemini-cli
```

macOS/Linux 加 `sudo`：

```bash
sudo npm install -g @google/gemini-cli
```

### 验证安装

```bash
gemini --version
```

---

## 登录 Google 账号

### 方式一：浏览器登录（推荐）

```bash
gemini
```

终端会提示打开浏览器，按 Enter 会自动打开。

### 方式二：使用 API Key

```bash
export GEMINI_API_KEY="AIxxx..."
gemini
```

API Key 在 [Google AI Studio](https://aistudio.google.com/apikey) 获取。

---

## 基础用法

### 启动对话

```bash
gemini
```

### 非交互模式

```bash
gemini -p "你好，帮我解释这段代码"
```

### 指定模型

```bash
gemini -m gemini-2.5-flash
```

---

## 卸载

```bash
npm uninstall -g @google/gemini-cli
```
