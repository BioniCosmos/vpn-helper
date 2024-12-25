# VPN Helper

A command-line utility to simplify and enhance your VPN connection experience

[中文](./README-zh.md)

## Is VPN Helper for You?

VPN Helper is the right tool if:

- You use a VPN for secure connectivity.
- Your VPN protocol is compatible with OpenConnect.
- Your VPN server password includes TOTP (Time-based One-Time Password).
- You want to avoid cluttering your route table with unnecessary VPN routes.

## Prerequisites

VPN Helper depends on the following programs:

- [OpenConnect](https://www.infradead.org/openconnect/)
- [vpn-slice](https://github.com/dlenski/vpn-slice)

For macOS users, you can easily install them via Homebrew:

```shellsession
$ brew install openconnect vpn-slice
```

## Configuration

Set up VPN Helper by creating a `.env` file in your working directory with the following structure:

```dotenv
USERNAME=
PASSWORD=
TOTP_SECRET=
HOST=
ROUTE_CIDR=
```

## Running Modes

VPN Helper provides two modes: **`once`** and **`watch`**.

### Mode: `once` (Default)

Start the VPN client and run it as a background daemon:

```shellsession
# vpn-helper connect
```

To stop the VPN client, use:

```shellsession
# vpn-helper disconnect
```

### Mode: `watch`

The `watch` mode is primarily used in conjunction with service managers, such as launchd on macOS.

To register VPN Helper as a launchd service, run:

```shellsession
# vpn-helper add-service
```

To unregister the service, run:

```shellsession
# vpn-helper remove-service
```

## Credits

Special thanks to Claude—you save the day!
