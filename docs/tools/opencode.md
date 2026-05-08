# OpenCode 安装教程

## 重要说明

**OpenCode 目前不支持 Windows**，仅支持 macOS 和 Linux。Windows 用户请跳过本教程。

---

## macOS / Linux

### 方式一：安装脚本（推荐）

打开终端，运行：

```bash
curl -fsSL https://opencode.ai/install | bash
```

脚本会自动：
1. 下载对应系统的最新版本
2. 解压到 `~/.opencode/bin/`
3. 提示你把路径加入 PATH

### 方式二：Homebrew（macOS）

```bash
brew install opencode-ai/tap/opencode
```

### 方式三：Go 安装

需要有 Go 1.24+：

```bash
go install github.com/opencode-ai/opencode@latest
```

### 方式四：AUR（Arch Linux）

```bash
yay -S opencode-ai-bin
```

---

## 安装后配置 PATH

安装脚本可能会提示需要手动添加 PATH。如果没有自动添加，手动添加：

### bash（大多数 Linux）

```bash
echo 'export PATH="$HOME/.opencode/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### zsh（macOS 默认 + modern Linux）

```bash
echo 'export PATH="$HOME/.opencode/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

### 验证

```bash
opencode --version
```

---

## 基础用法

### 启动对话

```bash
opencode
```

### 非交互模式

```bash
opencode -p "写一个 hello world 程序"
```

### 配置文件

OpenCode 配置文件在：
- `~/.opencode.json`
- 或 `$XDG_CONFIG_HOME/opencode/opencode.json`

---

## 卸载

### 方式一：删除二进制

```bash
rm -rf ~/.opencode
```

然后从 `~/.bashrc` 或 `~/.zshrc` 中移除 PATH 配置。

### 方式二：Homebrew

```bash
brew uninstall opencode
```

---

## Windows 替代方案

Windows 用户可以考虑：
- [Claude Code](./claude-code.md) — 支持 Windows，功能类似
- [VS Code + Copilot](https://code.visualstudio.com/docs/copilot/overview) — IDE 集成方案
