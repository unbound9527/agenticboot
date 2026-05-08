# 网络问题解决指南

AgenticBoot 需要访问 GitHub 和 npm 来下载安装工具。国内网络环境下推荐以下方案。

## 1. 使用代理 / VPN（推荐）

优先使用代理或 VPN，一劳永逸解决所有网络问题。

### Windows 客户端

| 项目 | 说明 |
|------|------|
| [2dust/v2rayN](https://github.com/2dust/v2rayN) | 最稳定的 Windows 客户端，支持 Xray + sing-box，老牌可靠 |
| [clash-verge-rev/clash-verge-rev](https://github.com/clash-verge-rev/clash-verge-rev) | 基于 Mihomo 内核，界面美观，规则路由强大 |
| [MatsuriDayo/nekoray](https://github.com/MatsuriDayo/nekoray) | Qt 跨平台客户端，基于 sing-box |

### 多平台

| 项目 | 说明 |
|------|------|
| [Hiddify/hiddify-next](https://github.com/hiddify/hiddify-next) | 全平台（Win/Mac/Linux/Android/iOS），专为受限网络优化，自动智能分流 |
| [SagerNet/sing-box](https://github.com/SagerNet/sing-box) | 通用代理核心平台，新一代协议首选 |
| [MetaCubeX/mihomo](https://github.com/MetaCubeX/mihomo) | Clash Meta 内核，基于规则的 Go 代理 |

### 辅助工具

| 项目 | 说明 |
|------|------|
| [tlanyan/ghproxy](https://github.com/tlanyan/ghproxy) | GitHub 文件加速代理，无需全局代理即可加速 GitHub 下载 |

配置好代理/VPN 后，**开启全局模式**，然后回到 AgenticBoot 点击刷新重新检测。

---

## 2. 配置 npm 国内镜像

如果不想使用 VPN，也可以单独为 npm 配置国内镜像：

```bash
npm config set registry https://registry.npmmirror.com
```

验证：

```bash
npm config get registry
```

---

## 3. 命令行代理配置

已有代理但只想让命令行工具走代理：

CMD：

```cmd
set HTTP_PROXY=http://127.0.0.1:端口
set HTTPS_PROXY=http://127.0.0.1:端口
```

Git 代理：

```bash
git config --global http.proxy http://127.0.0.1:端口
git config --global https.proxy http://127.0.0.1:端口
```

---

## 4. 验证网络连通性

浏览器依次打开以下地址，确认能访问：

- https://github.com
- https://www.npmjs.com
- https://www.youtube.com

能打开即表示网络已通，回到 AgenticBoot 点击刷新重新检测。

---

仍有问题？请在 [GitHub Issues](https://github.com/unbound9527/agenticboot/issues) 反馈。
