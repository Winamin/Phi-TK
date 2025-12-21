# Phi TK
![render](arts/phi-tklogo.png)

Phi-TK 是一个基于 Tauri + Vue 3 的谱面渲染工具。

## 功能特性

- 支持wav pcm32le 96khz无损音频
- CRF码率控制
- 铺面揭秘功能
- 跨平台支持（Windows、Linux、macOS）
- 现代化的用户界面，基于 Vue 3 + Vuetify
- 高性能渲染引擎
- 手序分配AI引擎

## 系统要求

### 通用要求
- Node.js 18+ 
- Rust 1.60+
- pnpm

### 平台特定要求
- **Windows**: Windows 10+ 
- **Linux**: 支持现代桌面发行版
- **macOS**: macOS 10.15+

## 安装步骤

### 1. 安装依赖

#### 安装 Node.js
从 [Node.js 官网](https://nodejs.org/) 下载并安装 Node.js 18 或更高版本。

#### 安装 Rust
从 [Rust 官网](https://rustup.rs/) 下载并安装 Rust。

#### 安装 pnpm
```bash
npm install -g pnpm
```

### 2. 克隆仓库
```bash
git clone https://github.com/Winamin/Phi-TK.git
cd Phi-TK
```

### 3. 安装项目依赖
```bash
pnpm install
```

## 编译构建

### 开发模式
```bash
pnpm tauri dev
```

### 生产构建
```bash
pnpm tauri build
```

构建完成后，可执行文件将位于：
- Windows: `src-tauri/target/release/Phi-TK.exe`
- Linux: `src-tauri/target/release/phi-tk`
- macOS: `src-tauri/target/release/bundle/macos/`

## 使用说明

1. 启动应用程序
2. 通过界面导入 Phigros 谱面文件
3. 配置渲染参数（音频质量、视频码率等）
4. 开始渲染过程
5. 导出渲染结果

## 项目结构

```
Phi-TK/
├── src/                    # Vue 前端源码
│   ├── components/         # Vue 组件
│   ├── router/            # 路由配置
│   └── assets/            # 静态资源
├── src-tauri/             # Tauri 后端源码
│   ├── src/               # Rust 源码
│   ├── assets/            # 应用资源
│   └── icons/             # 应用图标
└── arts/                  # 文档图片
```

## 开发脚本

- `pnpm dev` - 启动前端开发服务器
- `pnpm build` - 构建前端
- `pnpm type-check` - 类型检查
- `pnpm lint` - 代码检查和自动修复
- `pnpm prettier` - 代码格式化

## 许可证

本项目采用开源许可证，详见 LICENSE 文件。

## 贡献

欢迎提交 Issue 和 Pull Request 来改进这个项目。

## 相关链接

- [Phi-TK-render-lib](https://github.com/Winamin/Phi-TK-render-lib)
- [Tauri](https://tauri.app/)
- [Vue 3](https://vuejs.org/)