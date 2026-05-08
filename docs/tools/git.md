# Git 安装教程

## Windows

### 方式一：官网下载（推荐）

1. 打开 **https://git-scm.com/download/win**
2. 浏览器会自动下载对应版本
3. 双击运行 `.exe` 安装包
4. 安装选项中**全部保持默认**，直接一路点 Next
5. 安装完成后**打开一个新的 PowerShell 窗口**（不是 CMD），输入验证：

```powershell
git --version
```

看到类似以下输出即成功：

```
git version 2.51.0.windows.1
```

### 方式二：winget

```powershell
winget install Git.Git
```

### 方式三：Chocolatey

```powershell
choco install git -y
```

---

## macOS

### 方式一：Xcode Command Line Tools（最快）

打开 Terminal，输入：

```bash
xcode-select --install
```

弹出提示点"安装"即可。

### 方式二：Homebrew

```bash
brew install git
```

### 验证安装

```bash
git --version
```

---

## Linux

### Ubuntu / Debian

```bash
sudo apt update
sudo apt install git
```

### CentOS / RHEL

```bash
sudo yum install git
```

### Arch Linux

```bash
sudo pacman -S git
```

### 验证安装

```bash
git --version
```

---

## 安装后必做配置

不管哪个系统，安装完 Git 后第一件事是设置你的名字和邮箱：

```bash
git config --global user.name "你的名字"
git config --global user.email "你的邮箱@example.com"
```

这条配置是用来标注你提交代码时的身份，**没有对错，只要填就行**。

---

## 卸载

### Windows

1. 按 `Win + I` 打开"设置"
2. 进入"应用" → "已安装应用"
3. 搜索"Git"，点击卸载

### macOS

Git 如果是经由 Xcode Command Line Tools 安装的，**无法单独卸载**。如果是用 Homebrew 安装的：

```bash
brew uninstall git
```

### Linux

```bash
sudo apt remove git        # Debian/Ubuntu
sudo yum remove git        # CentOS/RHEL
sudo pacman -Rns git      # Arch Linux
```
