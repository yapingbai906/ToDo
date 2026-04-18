# DeskTodo

一个简洁优雅的 macOS 菜单栏待办事项工具。点击菜单栏图标或使用全局热键即可快速呼出。

## 功能特性

- **菜单栏访问** — 点击菜单栏图标呼出/隐藏窗口
- **全局快捷键** — 默认 `Ctrl + 2` 呼出/隐藏窗口
- **任务管理** — 添加、完成、删除任务
- **颜色标签** — 每条任务根据内容自动生成独特颜色
- **截止日期** — 支持为任务设置截止时间
- **过滤器** — 切换「全部 / 待办 / 已完成」视图
- **自定义热键** — 可在设置中修改全局快捷键
- **点击外部自动隐藏** — 点击窗口外区域或桌面自动收起

## 截图

<img src="https://raw.githubusercontent.com/yapingbai906/ToDo/main/demo.png" width="350" alt="DeskTodo 截图"/>

## 下载安装

1. **[Download ToDo_0.1.0_x64.dmg](https://github.com/yapingbai906/ToDo/releases/download/v0.1.0/ToDo_0.1.0_x64.dmg)**
2. 打开 DMG，将 **ToDo.app** 拖入应用程序
3. 首次运行需在「系统设置 → 隐私与安全性 → 安全性」中允许运行

## 使用方法

### 显示窗口
- **方式一**：点击菜单栏的 ToDo 图标
- **方式二**：按下全局快捷键 `Ctrl + 2`（默认）

### 隐藏窗口
- **方式一**：再次点击菜单栏图标
- **方式二**：再次按下 `Ctrl + 2`
- **方式三**：点击窗口外部区域（自动隐藏）
- **方式四**：点击桌面空白处（自动隐藏）

### 修改热键
1. 点击窗口右上角的设置图标
2. 点击「录制」按钮
3. 按下你想要的快捷键组合（如 `Ctrl + Shift + Space`）
4. 点击「保存」

> 热键需要至少一个修饰键（Ctrl / Option / Cmd / Shift）+ 一个按键

## 开发

### 环境要求

- **Rust** ≥ 1.70（用于 Tauri 后端编译）
- **Node.js** ≥ 18
- **pnpm**（或 npm / yarn）

### 安装步骤

```bash
# 1. 克隆项目
git clone https://github.com/yapingbai906/ToDo.git
cd ToDo

# 2. 安装前端依赖
pnpm install

# 3. 安装 Rust（如未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# 安装完成后执行以下命令加载环境变量
source ~/.cargo/env
```

### 开发命令

```bash
# 启动开发模式（前端热重载 + Rust 自动编译）
pnpm tauri dev

# 构建生产版本
pnpm tauri build

# 仅构建前端
pnpm build
```

### 项目结构

```
ToDo/
├── src/                          # Vue 前端源码
│   ├── App.vue                   # 主应用组件
│   ├── components/               # UI 组件
│   │   ├── TaskInput.vue         # 任务输入框
│   │   ├── TaskItem.vue          # 任务条目
│   │   ├── TaskList.vue          # 任务列表
│   │   └── SettingsPanel.vue     # 设置面板（热键设置）
│   ├── composables/              # 组合式函数
│   │   ├── useTasks.ts           # 任务状态管理
│   │   └── useConfig.ts          # 配置/快捷键管理
│   ├── types/
│   │   └── task.ts               # 任务类型定义
│   └── style.css                 # 全局样式
├── src-tauri/                    # Tauri/Rust 后端源码
│   ├── src/
│   │   ├── lib.rs                # 主逻辑（窗口/托盘/任务CRUD）
│   │   └── main.rs               # 入口
│   └── tauri.conf.json           # Tauri 配置
└── dist/                         # 前端构建产物
```

### 技术栈

| 层级 | 技术 |
|------|------|
| 前端框架 | Vue 3 + TypeScript |
| 构建工具 | Vite |
| 后端框架 | Tauri 2 (Rust) |
| 状态存储 | 本地 JSON 文件 |
| 快捷键 | tauri-plugin-global-shortcut |

### 数据存储

任务和配置保存在：
```
~/Library/Application Support/com.desktodo.desk/
├── tasks.json    # 任务数据
└── config.json   # 快捷键配置
```

## License

MIT
