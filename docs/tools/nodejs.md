# Node.js 安装教程

## Windows

### 方式一：官网下载（推荐）

1. 打开浏览器访问 **https://nodejs.org/**
2. 点击绿色的 **LTS（长期支持版）** 按钮下载
3. 下载完成后双击运行 `.msi` 安装包
4. 安装程序全程点"Next"即可，**最后一步取消勾选"Install additional tools"**（不需要，会拖慢安装）
5. 安装完成后**打开一个新的 PowerShell 窗口**，输入验证：

```powershell
node --version
npm --version
```

看到版本号即安装成功，例如：

```
v22.15.0
10.9.0
```

### 方式二：winget（Windows 10/11 自带）

打开 PowerShell（不要用 CMD），输入：

```powershell
winget install OpenJS.NodeJS.LTS
```

### 方式三：Chocolatey

```powershell
choco install nodejs-lts -y
```

---

## macOS

### 方式一：官网下载（推荐）

1. 打开 **https://nodejs.org/**
2. 下载 macOS 安装包（.pkg）
3. 双击运行安装包，全程点"继续"

### 方式二：Homebrew（推荐有 Homebrew 的用户）

```bash
brew install node
```

### 验证安装

```bash
node --version
npm --version
```

---

## Linux

### Ubuntu / Debian

```bash
curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
sudo apt-get install -y nodejs
```

### CentOS / RHEL

```bash
curl -fsSL https://rpm.nodesource.com/setup_22.x | sudo bash -
sudo yum install -y nodejs
```

### Arch Linux

```bash
sudo pacman -S nodejs npm
```

### 验证安装

```bash
node --version
npm --version
```

---

## 卸载

### Windows

**方式一：通过设置卸载**
1. 按 `Win + I` 打开"设置"
2. 进入"应用" → "已安装应用"
3. 搜索"Node.js"，点击卸载

**方式二：通过安装包卸载**
重新运行当初下载的 `.msi` 文件，选择"Remove"

**方式三：winget**

```powershell
winget uninstall OpenJS.NodeJS.LTS
```

### macOS

```bash
sudo rm -rf /usr/local/lib/node_modules /usr/local/bin/node /usr/local/bin/npm
```

或如果你用 Homebrew 安装：

```bash
brew uninstall node
```

### Linux

```bash
sudo apt remove nodejs npm        # Debian/Ubuntu
sudo yum remove nodejs npm        # CentOS/RHEL
sudo pacman -Rns nodejs npm       # Arch Linux
```
