# OpenClaw 安装教程

## 重要说明

**OpenClaw 不是专门的编程工具**，而是一个个人 AI 助手，支持接入 WhatsApp、Telegram、Discord 等多个消息平台。

如果你是在找 AI 编程工具，请参考：
- [Claude Code](./claude-code.md)
- [Codex](./codex.md)
- [Gemini CLI](./gemini-cli.md)

---

## 前置要求

**必须先安装 Node.js（推荐 Node 24，或 22.14+）**。

验证：

```bash
node --version
```

未安装先看 [Node.js 安装教程](./nodejs.md)。

---

## Windows / macOS / Linux（通用）

### 安装

```bash
npm install -g openclaw@latest
```

macOS/Linux 加 `sudo`：

```bash
sudo npm install -g openclaw@latest
```

### 验证安装

```bash
openclaw --version
```

---

## 首次配置

### 运行引导向导

```bash
openclaw onboard --install-daemon
```

向导会引导你：
1. 配置 API Key（需要 OpenAI 或 Anthropic API Key）
2. 选择要连接的消息平台（Telegram / Discord / WhatsApp 等）
3. 设置通知偏好

### 配置 API Key

编辑 `~/.openclaw/openclaw.json`：

```json
{
  "agent": {
    "model": "anthropic/claude-3-5-sonnet"
  },
  "providers": {
    "openai": {
      "apiKey": "sk-xxxxx"
    }
  }
}
```

---

## 常用命令

### 启动 Gateway

```bash
openclaw gateway --port 18789 --verbose
```

### 发送消息

```bash
openclaw message send --target 用户ID --message "你好"
```

### 检查状态

```bash
openclaw doctor
```

---

## 卸载

```bash
npm uninstall -g openclaw
```
