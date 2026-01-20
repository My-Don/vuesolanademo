# vuesolanademo

<p align="center">
  <strong>Vue 3 + Vite + Solana（WSL2）开发环境示例项目</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Vue-3.x-42b883" />
  <img src="https://img.shields.io/badge/Vite-latest-646cff" />
  <img src="https://img.shields.io/badge/Solana-CLI-14f195" />
  <img src="https://img.shields.io/badge/Platform-Windows%2011%20%2B%20WSL2-blue" />
</p>

---

## 📌 项目简介

`vuesolanademo` 是一个用于演示 **Vue 3 + Vite 前端项目** 与 **Solana 智能合约开发环境（WSL2 + Ubuntu）** 的完整示例。

适合人群：

* Solana 初学者
* Web3 / DApp 前端开发者
* Windows 用户（不想直接用原生 Linux）

---

## 🧱 技术栈

* **前端**：Vue 3、Vite
* **区块链**：Solana、Anchor
* **运行环境**：Windows 11 + WSL2 (Ubuntu)
* **工具链**：Node.js、Yarn、Rust
* **IDE**：VS Code / Google Antigravity

---

## 📂 项目结构（示例）

```
vuesolanademo/
├─ src/                # Vue 前端源码
├─ public/
├─ package.json
├─ vite.config.ts
├─ README.md
└─ docs/               # 开发环境 & 教程文档（可扩展）
```

---

## 🚀 快速开始（Vue 前端）

### 1️⃣ 安装依赖

```bash
npm install
```

### 2️⃣ 本地开发

```bash
npm run dev
```

### 3️⃣ 构建生产版本

```bash
npm run build
```

---

## 🛠️ 开发环境准备（Windows 11 + WSL2）

### 一、开启 CPU 虚拟化（BIOS）

> ⚠️ WSL2 必须开启虚拟化

* 开机按 `Del` / `F2` 进入 BIOS
* `Advanced → CPU Configuration`

**Intel CPU**

```
Intel Virtualization Technology → Enabled
Intel VT-d → Enabled（可选）
```

**AMD CPU**

```
SVM Mode → Enabled
```

保存并退出（F10）。

---

### 二、安装 WSL2

#### PowerShell（管理员）

```powershell
wsl --install
```

查看可安装系统：

```powershell
wsl --list --online
```

安装 Ubuntu：

```powershell
wsl --install -d Ubuntu
```

---

### 三、WSL 网络 & 代理配置（推荐）

#### `.wslconfig`

路径：

```
C:\Users\Administrator\.wslconfig
```

```ini
[wsl2]
networkingMode = mirrored
autoProxy = true
```

#### Ubuntu 中设置代理

```bash
export http_proxy="http://127.0.0.1:7890"
export https_proxy="http://127.0.0.1:7890"
```

---

## ⛓️ Solana 开发环境安装（WSL Ubuntu）

### 一键安装（官方集成脚本）

```bash
curl --proto '=https' --tlsv1.2 -sSfL https://solana-install.solana.workers.dev | bash
```

### 永久配置 PATH

```bash
echo 'export
```
