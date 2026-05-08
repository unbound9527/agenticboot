# Claude Code 安装教程

## 前置要求

安装 Claude Code 之前，**必须先安装 Node.js**。

打开终端验证是否已有 Node.js：

```bash
node --version
npm --version
```

如果显示"command not found"，先参照 [Node.js 安装教程](./nodejs.md) 安装 Node.js。

---

## Windows / macOS / Linux（通用）

Claude Code 通过 npm 安装，**所有系统命令相同**。

### 安装

打开终端，运行：

```bash
npm install -g @anthropic-ai/claude-code
```

macOS/Linux 可能需要加 `sudo`：

```bash
sudo npm install -g @anthropic-ai/claude-code
```

### 验证安装

```bash
claude --version
```

---

## 首次登录

Claude Code 安装完成后，打开终端输入：

```bash
claude
```

终端会显示：

```
Authenticate with Anthropic by visiting:
https://auth.anthropic.com/...
```

按 `Enter` 会自动用默认浏览器打开链接，在浏览器中完成登录即可。

### 如果没有自动打开浏览器

手动复制终端中显示的 URL，粘贴到浏览器打开。

---

## 使用 API Key（不登录浏览器）

如果你不想用浏览器登录，可以直接设置 API Key：

### Windows (PowerShell)

```powershell
$env:ANTHROPIC_API_KEY="sk-ant-xxxxx"
claude
```

### macOS / Linux

```bash
export ANTHROPIC_API_KEY="sk-ant-xxxxx"
claude
```

API Key 在 [Anthropic Console](https://console.anthropic.com/) 获取。

---

## 基础用法

### 启动对话

```bash
claude
```

### 指定模型

```bash
claude -m claude-opus-4-7
claude -m claude-sonnet-4-6
```

### 非交互模式（一次提问）

```bash
claude -p "解释这段代码的作用"
```

### 退出

输入 `exit` 或按 `Ctrl + C`。

---

## 卸载

```bash
npm uninstall -g @anthropic-ai/claude-code
```
