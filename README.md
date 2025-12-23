# vuesolanademo

This template should help get you started developing with Vue 3 in Vite.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Vue (Official)](https://marketplace.visualstudio.com/items?itemName=Vue.volar) (and disable Vetur).

## Recommended Browser Setup

- Chromium-based browsers (Chrome, Edge, Brave, etc.):
  - [Vue.js devtools](https://chromewebstore.google.com/detail/vuejs-devtools/nhdogjmejiglipccpnnnanhbledajbpd) 
  - [Turn on Custom Object Formatter in Chrome DevTools](http://bit.ly/object-formatters)
- Firefox:
  - [Vue.js devtools](https://addons.mozilla.org/en-US/firefox/addon/vue-js-devtools/)
  - [Turn on Custom Object Formatter in Firefox DevTools](https://fxdx.dev/firefox-devtools-custom-object-formatters/)

## Customize configuration

See [Vite Configuration Reference](https://vite.dev/config/).

## Project Setup

```sh
npm install
```

### Compile and Hot-Reload for Development

```sh
npm run dev
```

### Compile and Minify for Production

```sh
npm run build
```
### windows 11安装wsl, 安装solana依赖, 配置wsl插件
先确认windows11是否开启了虚拟化,没开的话，操作如下
华硕主板的 BIOS/UEFI 中开启虚拟化技术（通常指 Intel VT-x 或 AMD-V）是一个很常见的操作。以下是详细步骤，适用于大多数现代华硕主板。

核心步骤总览
进入 BIOS/UEFI 设置界面。

找到与虚拟化相关的选项（通常在 高级 或 CPU 配置 菜单中）。

将虚拟化选项设置为 “开启”。

保存并退出 BIOS。

详细图文步骤
第一步：进入 BIOS/UEFI 设置
重启或开机。

在自检（POST）画面出现时，立即并连续地按键盘上的 Del 键或 F2 键（绝大多数华硕主板使用这两个键，开机画面上通常也会有提示）。

成功进入 BIOS 界面，界面通常是图形化的（UEFI BIOS）。

提示：如果你的系统是 Windows 10/11 且启用了“快速启动”，可能难以按出BIOS。可以在系统内：

打开「设置」->「更新与安全」->「恢复」->「高级启动」下的“立即重新启动” -> 「疑难解答」-> 「高级选项」-> 「UEFI固件设置」-> 「重启」。

第二步：找到虚拟化选项（关键）
BIOS 界面有多种模式，通常按 F7 可以在“简易模式”和“高级模式”之间切换。请进入“高级模式”。

虚拟化选项的路径主要有两种，取决于主板和BIOS版本：

路径 A（最常见）：

进入 Advanced（高级） 选项卡。

选择 CPU Configuration（CPU 配置）。

向下滚动，找到与虚拟化技术相关的选项，例如：

对于 Intel CPU： Intel (R) Virtualization Technology 或 Intel VT-x。将其设置为 Enabled。

可能还有一个 Intel VT-d（用于直接I/O虚拟化），如果看到，也可以一并开启。

对于 AMD CPU： SVM Mode（安全虚拟机模式）。将其设置为 Enabled。

路径 B（另一种可能）：

在 Advanced（高级） 选项卡下。

找到 System Agent Configuration（系统代理配置） 或 North Bridge。

进入后，找到 VT-d 或类似的虚拟化选项，将其设置为 Enabled。

注意：在一些老主板或特定系列的BIOS中，选项也可能在 Advanced -> CPU Configuration -> Secure Virtual Machine Mode（AMD）中。

第三步：保存并退出
按 F10 键，这是保存更改并退出的通用快捷键。

会弹出确认窗口，询问是否保存更改并重置。

选择 Yes 或按 Enter 确认。

电脑将自动重启。

验证是否开启成功
重启进入 Windows 后，可以通过以下方法验证：

任务管理器查看：

按 Ctrl + Shift + Esc 打开任务管理器。

切换到 “性能” 选项卡。

点击 “CPU”，在右下角查看 “虚拟化” 状态。如果显示 “已启用”，则表示成功。

使用命令行验证（Intel CPU）：

以管理员身份打开“命令提示符”或“PowerShell”。

输入命令：systeminfo

在输出信息中查找 “Hyper-V 要求” 部分，如果看到 “虚拟机监视器模式扩展: 是”，则说明虚拟化已启用。


powershell管理员模式下打开终端

设置powershell终端临时HTTP/HTTPS 代理
```sh
$env:HTTP_PROXY="http://127.0.0.1:7890"
$env:HTTPS_PROXY="http://127.0.0.1:7890"
```

```sh
wsl --install
```

查看可安装的ubuntu版本
```sh
wsl --list --online
```

安装ubuntu
```sh
wsl --install -d <DistroName>
```
powershell终端关闭wsl
```sh
wsl --shutdown
```

配置.wslconfig
打开C盘，路径C:\Users\Administrator,添加一个.wslconfig文件并保存,内容如下
```sh
[wsl2]
# 网络设置
networkingMode = mirrored
autoProxy = true
```
打开ubuntu子系统即wsl终端,输入以下命令
```sh
export http_proxy="http://127.0.0.1:7890"
export https_proxy="http://127.0.0.1:7890"
```
安装solana相关的依赖
```sh
curl --proto '=https' --tlsv1.2 -sSfL https://solana-install.solana.workers.dev | bash
```
控制台打印数据如下即安装成功:
```sh
Installed Versions:
Rust: rustc 1.91.1 (ed61e7d7e 2025-11-07)
Solana CLI: solana-cli 3.0.10 (src:96c3a851; feat:3604001754, client:Agave)
Anchor CLI: anchor-cli 0.32.1
Surfpool CLI: surfpool 0.12.0
Node.js: v24.10.0
Yarn: 1.22.1
```

永久保存配置
```sh
echo 'export PATH="/root/.local/share/solana/install/active_release/bin:$PATH" ' >> ~/.bashrc
source ~/.bashrc
```

检查安装的依赖
```sh
rustc --version && solana --version && anchor --version && surfpool --version && node --version && yarn --version
```

安装wsl插件到vscode,请查看如下网址
```sh
https://code.visualstudio.com/docs/remote/wsl-tutorial
```

