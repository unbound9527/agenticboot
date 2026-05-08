# 网络问题解决指南

AgenticBoot 需要访问 GitHub 和 npm 来下载安装工具。如果你在国内网络环境下遇到连接问题，以下是常见的解决方法。

## 1. 配置 npm 国内镜像

打开终端（CMD 或 PowerShell），执行：

```bash
npm config set registry https://registry.npmmirror.com
```

验证是否生效：

```bash
npm config get registry
```

## 2. 配置 Git 代理

如果你有 VPN 或 HTTP 代理，配置 Git 走代理：

```bash
git config --global http.proxy http://127.0.0.1:你的代理端口
git config --global https.proxy http://127.0.0.1:你的代理端口
```

取消代理：

```bash
git config --global --unset http.proxy
git config --global --unset https.proxy
```

## 3. 设置系统环境变量代理

CMD：

```cmd
set HTTP_PROXY=http://127.0.0.1:端口
set HTTPS_PROXY=http://127.0.0.1:端口
```

PowerShell：

```powershell
$env:HTTP_PROXY="http://127.0.0.1:端口"
$env:HTTPS_PROXY="http://127.0.0.1:端口"
```

## 4. 系统代理设置

- Windows 设置 → 网络和 Internet → 代理
- 确保"自动检测设置"已开启，或手动配置代理地址

## 5. 使用 VPN

如果以上方法都不行，建议使用 VPN 或网络加速器（如 Clash、V2Ray 等），开启全局模式后重试。

## 6. 验证网络连通性

依次在浏览器中打开以下地址，确认能够访问：

- https://github.com
- https://www.npmjs.com
- https://www.youtube.com

能打开即表示网络已通，回到 AgenticBoot 点击刷新重新检测。

---

仍有问题？请在 [GitHub Issues](https://github.com/unbound9527/agenticboot/issues) 反馈。
