# Codex 安装教程

## 前置要求

安装 Codex 之前，**必须先安装 Node.js**。

验证 Node.js 是否已安装：

```bash
node --version
npm --version
```

没有显示版本号的话，先参照 [Node.js 安装教程](./nodejs.md)。

---

## Windows / macOS / Linux（通用）

### 安装

```bash
npm install -g @openai/codex
```

macOS/Linux 可能需要加 `sudo`：

```bash
sudo npm install -g @openai/codex
```

### 验证安装

```bash
codex --version
```

---

## 登录 OpenAI 账号

Codex 需要登录 OpenAI 才能使用。

### 方式一：终端登录

```bash
codex
```

终端会提示你打开浏览器完成授权。

### 方式二：设置 API Key

```bash
export OPENAI_API_KEY="sk-xxxxx"
```

把 `sk-xxxxx` 换成你从 [OpenAI Platform](https://platform.openai.com/api-keys) 获取的真实 Key。

---

## 基础用法

### 启动对话

```bash
codex
```

### 非交互模式

```bash
codex --message "写一个快速排序函数"
```

---

## 卸载

```bash
npm uninstall -g @openai/codex
```
