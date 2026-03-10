# xfw-platform

Wayland 平台層，負責窗口管理、事件循環、輸入處理和緩冲區管理。

## 模塊結構

```
crates/xfw-platform/src/
├── lib.rs          # 主入口，PlatformSurface 導出
├── connection.rs   # Wayland 連接管理
├── surface.rs      # Layer Shell 表面支援
├── xdg.rs          # XDG 窗口支援
├── buffer.rs       # SHM Buffer 池管理
├── event_loop.rs   # mio/epoll 事件循環
├── input.rs        # 輸入事件處理
├── clipboard.rs    # 剪貼簿支援
└── error.rs        # 錯誤類型定義
```

## 核心類型

### PlatformSurface

主入口結構，管理 Wayland 連接和所有表面。

```rust
pub struct PlatformSurface {
    connection: WaylandConnection,
    event_loop: EventLoop,
    surface_manager: SurfaceManager,
    input_manager: InputManager,
    buffer_pools: HashMap<u32, BufferPool>,
    // ...
}
```

#### 方法

| 方法 | 說明 |
|------|------|
| `new()` | 創建新的 PlatformSurface，連接到 Wayland |
| `dispatch_loop()` | 進入事件循環，阻塞直到退出 |
| `create_layer_surface(config)` | 創建 Layer Shell 表面 |
| `set_layer_surface_size(id, w, h)` | 設置表面尺寸 |
| `commit_layer_surface(id)` | 提交表面更改 |
| `get_buffer(id)` | 獲取可用 Buffer |
| `attach_buffer(id, buffer)` | 附加 Buffer 到表面 |
| `get_all_surfaces()` | 獲取所有表面幾何信息 |
| `get_input_surface_under_cursor(x, y)` | 命中測試 |
| `quit()` | 退出事件循環 |

## Layer Shell 表面

### LayerSurfaceConfig

```rust
pub struct LayerSurfaceConfig {
    pub anchor: Anchor,              // 錨點位置
    pub layer: Layer,                // 層級
    pub keyboard_interactivity: KeyboardInteractivity,
    pub margin: (i32, i32, i32, i32), // 邊距 (上, 右, 下, 左)
    pub namespace: String,           // 命名空間
    pub width: u32,
    pub height: u32,
}
```

### Anchor

```
┌──────────────────────────────────────┐
│ TopLeft    │      Top      │TopRight │
├────────────┼──────────────┼──────────┤
│            │              │          │
│   Left     │    All       │  Right   │
│            │              │          │
├────────────┼──────────────┼──────────┤
│BottomLeft  │   Bottom     │BtmRight  │
└──────────────────────────────────────┘
```

```rust
enum Anchor {
    Top, Bottom, Left, Right,
    TopLeft, TopRight, BottomLeft, BottomRight,
    All,
}
```

### Layer

```rust
enum Layer {
    Background,  // 背景層
    Bottom,      // 底部層
    Top,         // 頂部層 (常用於狀態欄)
    Overlay,     // 覆蓋層
}
```

### 使用範例

```rust
let mut platform = PlatformSurface::new()?;

let config = LayerSurfaceConfig {
    anchor: Anchor::Top,
    layer: Layer::Top,
    keyboard_interactivity: KeyboardInteractivity::None,
    margin: (0, 0, 0, 0),
    namespace: "my-widget".to_string(),
    width: 800,
    height: 40,
};

let surface_id = platform.create_layer_surface(config)?;
platform.set_layer_surface_size(surface_id, 800, 40)?;
platform.commit_layer_surface(surface_id)?;

platform.dispatch_loop()?;
```

## XDG 窗口

### XdgWindowConfig

```rust
pub struct XdgWindowConfig {
    pub title: String,
    pub app_id: Option<String>,
    pub width: u32,
    pub height: u32,
    pub min_width: u32,
    pub min_height: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub decorations: bool,
    pub resizable: bool,
    pub fullscreen: bool,
    pub maximized: bool,
    pub minimized: bool,
    pub focus: bool,
}
```

### 建構器方法

```rust
let config = XdgWindowConfig::new("My App", 1024, 768)
    .with_app_id("com.example.app")
    .with_min_size(400, 300)
    .with_max_size(1920, 1080)
    .with_decorations(true)
    .with_resizable(true)
    .fullscreen(false)
    .maximized(false);
```

### XdgWindow 方法

| 方法 | 說明 |
|------|------|
| `set_size(w, h)` | 設置窗口尺寸 |
| `set_title(title)` | 設置窗口標題 |
| `set_app_id(id)` | 設置應用 ID |
| `set_fullscreen(bool)` | 切換全屏 |
| `set_maximized(bool)` | 切換最大化 |
| `set_minimized()` | 最小化 |
| `start_resize(edge)` | 開始調整大小 |
| `move_()` | 開始移動 |
| `commit()` | 提交更改 |

## Buffer 管理

### BufferConfig

```rust
pub struct BufferConfig {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: u32,  // wl_shm::Format
}
```

```rust
let config = BufferConfig::new(1920, 1080)
    .with_stride(7680)  // 自定義 stride
    .with_format(wl_shm::Format::Argb8888);
```

### BufferPool

Buffer 池化管理，自動復用閒置 Buffer：

```rust
let mut pool = BufferPool::new(shm, BufferConfig::new(800, 600));

// 獲取可用 Buffer
let buffer = pool.acquire(qh)?;

// 使用 Buffer 進行渲染
let data = buffer.data();
// ... 渲染操作 ...

// 釋放 Buffer
pool.release(buffer.id);
```

## 事件循環

### EventLoop (mio)

基於 mio 的事件循環，實現零 CPU 佔用的 idle 等待：

```rust
let mut event_loop = EventLoop::new()?;

// 註冊文件描述符
event_loop.register_fd(fd, MyDispatcher);

// 運行事件循環
event_loop.run(dispatcher, Some(Duration::from_millis(100)))?;
```

### Timer

```rust
// 單次定時器
let timer = Timer::new(Duration::from_secs(1));

// 重複定時器
let mut repeating = Timer::repeating(Duration::from_millis(100));

if repeating.is_ready() {
    // 處理定時事件
    repeating.reset();
}
```

## 輸入處理

### InputManager

```rust
let mut manager = InputManager::new();

// 註冊表面映射
manager.register_surface(surface_id, wl_surface_id);

// 命中測試
let surfaces = vec![
    (id1, x1, y1, w1, h1),
    (id2, x2, y2, w2, h2),
];
if let Some(hit_id) = manager.hit_test(mouse_x, mouse_y, &surfaces) {
    // 命中的表面
}
```

### InputEvent

```rust
enum InputEvent {
    Pointer(PointerEvent),
    Keyboard(KeyboardEvent),
    PointerEnter { surface_id: u32, x: f64, y: f64 },
    PointerLeave { surface_id: u32 },
}
```

## PlatformEvent

平台層發送的事件，供 runtime 訂閱：

```rust
enum PlatformEvent {
    PointerEnter { surface_id: u32, x: f64, y: f64 },
    PointerLeave { surface_id: u32 },
    PointerMove { surface_id: u32, x: f64, y: f64 },
    PointerButton { surface_id: u32, button: u32, pressed: bool },
    PointerAxis { surface_id: u32, horizontal: f64, vertical: f64 },
    Keyboard { surface_id: u32, key: u32, pressed: bool },
    ConfigChanged { width: u32, height: u32 },
    Wake,
    Quit,
}
```

## 依賴項

```toml
[dependencies]
anyhow.workspace = true
parking_lot.workspace = true
serde.workspace = true
serde_json.workspace = true
smithay-client-toolkit.workspace = true
tracing.workspace = true
mio.workspace = true
wayland-client.workspace = true
wayland-protocols.workspace = true

[target.'cfg(unix)'.dependencies]
libc.workspace = true
```

## 系統需求

- Linux 系統
- Wayland 會話
- 依賴 `libxkbcommon-dev` (編譯時)

```bash
# Ubuntu/Debian
sudo apt install libxkbcommon-dev pkg-config
```

## 測試

```bash
# 運行所有測試
cargo test -p xfw-platform

# 運行特定模塊測試
cargo test -p xfw-platform buffer_tests
cargo test -p xfw-platform xdg_tests
```