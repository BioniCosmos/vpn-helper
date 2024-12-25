# VPN Helper

快速、自动地连接到 VPN

## VPN Helper 有什么用？

如果符合以下情况，VPN Helper 就能帮到你：

- 你正在使用 VPN。
- VPN 协议与 OpenConnect 兼容。
- VPN 账户密码包含 TOTP。
- 不想让路由表遭到污染。

## 依赖

VPN Helper 依赖以下程序：

- [OpenConnect](https://www.infradead.org/openconnect/)
- [vpn-slice](https://github.com/dlenski/vpn-slice)

macOS 用户可以通过 Homebrew 轻松安装：

```shellsession
$ brew install openconnect vpn-slice
```

## 配置

只需在工作目录创建以下结构的 `.env` 文件即可：

```dotenv
USERNAME=
PASSWORD=
TOTP_SECRET=
HOST=
ROUTE_CIDR=
```

## 运行模式

VPN Helper 提供两种模式：**`once`** 和 **`watch`**。

### 模式：`once`（默认）

启动 VPN 客户端并将其作为后台守护进程运行：

```shellsession
# vpn-helper connect
```

要停止 VPN 客户端，使用：

```shellsession
# vpn-helper disconnect
```

### 模式：`watch`

`watch` 模式主要与服务管理器一起搭配使用，如 macOS 的 launchd。

要将 VPN Helper 注册为 launchd 服务，运行：

```shellsession
# vpn-helper add-service
```

要注销该服务，运行：

```shellsession
# vpn-helper remove-service
```

## 致谢

Claude 真 🐮🍺 好吧。
