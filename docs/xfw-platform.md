# xfw-platform

Wayland 平台層，負責窗口管理、事件循環、輸入處理和緩冲區管理。

## 模塊結構

```
crates/xfw-platform/src/
├── lib.rs          # 主入口，PlatformSurface 導出
├── connection.rs   # Wayland 連接管理 (wayland-client 0.31)
├── surface.rs      # Layer Shell 表面支援
├── xdg.rs          # XDG 窗口支援
├── buffer.rs       # SHM Buffer 池管理
├── event_loop.rs   # mio 1.x 事件循環
├── input.rs        # 輸入事件處理
├── clipboard.rs    # 剪貼簿支援
└── error.rs        # 錯誤類型定義
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
memmap2 = "0.9"
mio.workspace = true
wayland-client.workspace = true
wayland-protocols.workspace = true
wayland-protocols-wlr = "0.3"

[target.'cfg(unix)'.dependencies]
libc = "0.2"
```

## 系統需求

- Linux 系統
- Wayland 會話
- 依賴 `libxkbcommon-dev` (編譯時)

```bash
# Ubuntu/Debian
sudo apt install libxkbcommon-dev pkg-config
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
    event_receiver: Option<Receiver<PlatformEvent>>,
    surface_geometries: HashMap<u32, SurfaceGeometry>,
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
| `get_surface_geometry(id)` | 獲取指定表面幾何信息 |
| `get_input_surface_under_cursor(x, y)` | 命中測試 |
| `poll_events()` | 獲取待處理的事件 |
| `roundtrip()` | 執行 Wayland roundtrip |
| `quit()` | 退出事件循環 |

### SurfaceGeometry

```rust
#[derive(Debug, Clone, Copy)]
pub struct SurfaceGeometry {
    pub x: f32,
    pub y: f32,
    pub width: u32,
    pub height: u32,
}
```

## Layer Shell 表面

### LayerSurfaceConfig

```rust
#[derive(Clone)]
pub struct LayerSurfaceConfig {
    pub anchor: Anchor,                        // 錨點位置
    pub layer: Layer,                          // 層級
    pub keyboard_interactivity: KeyboardInteractivity,
    pub margin: (i32, i32, i32, i32),          // 邊距 (上, 右, 下, 左)
    pub namespace: String,                     // 命名空間
    pub width: u32,
    pub height: u32,
}

impl Default for LayerSurfaceConfig {
    fn default() -> Self {
        Self {
            anchor: Anchor::Top,
            layer: Layer::Top,
            keyboard_interactivity: KeyboardInteractivity::None,
            margin: (0, 0, 0, 0),
            namespace: "xfw".to_string(),
            width: 0,
            height: 0,
        }
    }
}
```

### Anchor

```
┌──────────────────────────────────────┐
│ TopLeft      │ Top      │ TopRight   │
├──────────────┼──────────┼────────────┤
│              │          │            │
│    Left      │   All    │   Right    │
│              │          │            │
├──────────────┼──────────┼────────────┤
│ BottomLeft   │ Bottom   │ BtmRight   │
└──────────────────────────────────────┘
```

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    All,
}

impl Anchor {
    /// 轉換為 wayland 協議的 Anchor 類型
    pub fn to_wl(self) -> zwlr_layer_surface_v1::Anchor
}
```

### Layer

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Background,  // 背景層
    Bottom,      // 底部層
    Top,         // 頂部層 (常用於狀態欄)
    Overlay,     // 覆蓋層
}
```

### KeyboardInteractivity

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardInteractivity {
    None,      // 不請求鍵盤
    Exclusive, // 請求排他鍵盤焦點
    OnDemand,  // 請求焦點（按下時）
}
```

### 使用範例

```rust
use xfw_platform::{PlatformSurface, surface::{Anchor, Layer, LayerSurfaceConfig, KeyboardInteractivity}};

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
#[derive(Clone)]
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

### WindowResizeEdge

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowResizeEdge {
    None,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}
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
| `start_resize(edge)` | 開始調整大小 (當前為空操作) |
| `move_()` | 開始移動 (當前為空操作) |
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

impl BufferConfig {
    pub fn new(width: u32, height: u32) -> Self
    pub fn with_stride(self, stride: u32) -> Self
    pub fn with_format(self, format: u32) -> Self
    pub fn size(&self) -> usize
}
```

```rust
let config = BufferConfig::new(1920, 1080)
    .with_stride(7680)  // 自定義 stride
    .with_format(wl_shm::Format::Xrgb8888 as u32);
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

// 調整 Buffer 大小
pool.resize(width, height, qh)?;
```

### ShmBuffer

```rust
pub struct ShmBuffer {
    pub id: u32,
    pub buffer: wl_buffer::WlBuffer,
    pub config: BufferConfig,
    // 內部字段...
}

impl ShmBuffer {
    pub fn data(&mut self) -> &mut [u8]
    pub fn set_in_use(&mut self, in_use: bool)
    pub fn is_in_use(&self) -> bool
    pub fn width(&self) -> u32
    pub fn height(&self) -> u32
    pub fn stride(&self) -> u32
}
```

## 事件循環 (mio 1.x)

基於 mio 的事件循環，實現零 CPU 佔用的 idle 等待：

```rust
let mut event_loop = EventLoop::new()?;

// 註冊文件描述符
event_loop.register_fd(fd, MyDispatcher);

// 運行事件循環
event_loop.run(dispatcher, Some(Duration::from_millis(100)))?;
```

### EventLoop 方法

```rust
impl EventLoop {
    pub fn new() -> Result<Self>
    pub fn register<S>(&mut self, source: S, dispatcher: impl EventDispatcher + 'static) -> Token
    pub fn register_fd(&mut self, fd: RawFd, dispatcher: impl EventDispatcher + 'static) -> Token
    pub fn unregister(&mut self, token: Token)
    pub fn wake(&self)
    pub fn run<D>(&mut self, dispatcher: &mut D, timeout: Option<Duration>) -> Result<()>
    pub fn stop(&mut self)
}
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

### EventDispatcher trait

```rust
pub trait EventDispatcher: Send {
    fn dispatch(&mut self, source: &dyn EventSource, event: &Event);
}
```

### EventSource trait

```rust
pub trait EventSource: Send + Sync {
    fn fd(&self) -> RawFd;
    fn ready(&self, event: &Event, dispatcher: &mut dyn EventDispatcher);
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
pub enum InputEvent {
    Pointer(PointerEvent),
    Keyboard(KeyboardEvent),
    PointerEnter { surface_id: u32, x: f64, y: f64 },
    PointerLeave { surface_id: u32 },
}
```

### PointerEvent

```rust
pub struct PointerEvent {
    pub x: f64,
    pub y: f64,
    pub button: u32,
    pub state: PointerState,
}
```

### KeyboardEvent

```rust
pub struct KeyboardEvent {
    pub key: u32,
    pub state: KeyState,
}
```

## PlatformEvent

平台層發送的事件，供 runtime 訂閱：

```rust
#[derive(Debug, Clone)]
pub enum PlatformEvent {
    PointerEnter { surface_id: u32, x: f64, y: f64 },
    PointerLeave { surface_id: u32 },
    PointerMove { surface_id: u32, x: f64, y: f64 },
    PointerButton { surface_id: u32, button: u32, pressed: bool },
    PointerAxis { surface_id: u32, horizontal: f64, vertical: f64 },
    Keyboard { surface_id: u32, key: u32, pressed: bool },
    Keymap { fd: RawFd, size: u32 },
    ConfigChanged { width: u32, height: u32 },
    DataReceived { surface_id: u32, data: Vec<u8> },
    Wake,
    Quit,
}
```

### PlatformEventHandler trait

```rust
pub trait PlatformEventHandler: Send {
    fn handle_event(&mut self, event: PlatformEvent);
}
```

## Wayland 連接管理

### WaylandConnection

```rust
pub struct WaylandConnection {
    connection: Connection,
    event_queue: EventQueue<WaylandDispatcher>,
    globals: GlobalList,
    state: Arc<Mutex<WaylandState>>,
    fd: RawFd,
}

impl WaylandConnection {
    pub fn new() -> Result<Self>
    pub fn fd(&self) -> RawFd
    pub fn roundtrip(&mut self) -> Result<()>
    pub fn state(&self) -> Arc<Mutex<WaylandState>>
    pub fn get_surface(&self) -> Result<wl_surface::WlSurface>
    pub fn queue(&self) -> QueueHandle<WaylandDispatcher>
}
```

### WaylandState

```rust
pub struct WaylandState {
    pub display: wl_display::WlDisplay,
    pub compositor: wl_compositor::WlCompositor,
    pub subcompositor: wl_subcompositor::WlSubcompositor,
    pub shm: wl_shm::WlShm,
    pub layer_shell: Option<ZwlrLayerShellV1>,
    pub xdg_wm_base: Option<XdgWmBase>,
    pub registry: HashMap<String, Global>,
}
```

## API 更新說明 (wayland-client 0.31, mio 1.x)

### wayland-client 0.31 變更

1. **Dispatch trait 簽名變更**:
   - 第一個參數從 `&mut WaylandState` 改為 `&mut WaylandDispatcher`
   - `QueueHandle` 的泛型參數改為 `WaylandDispatcher`

2. **創建對象時需要傳入 user data**:
   ```rust
   // 舊 API
   shm.create_pool(fd, size, qh)
   
   // 新 API (0.31)
   shm.create_pool(borrowed_fd, size, qh, ())
   ```

3. **方法調用不再返回 Result**:
   ```rust
   // 舊 API
   surface.commit().map_err(...)
   
   // 新 API (0.31)
   surface.commit()
   ```

### mio 1.x 變更

1. **註冊方式變更**:
   ```rust
   // 舊 API
   poll.register(source_fd, token, Interest::READABLE | Interest::WRITABLE)
   
   // 新 API (1.x)
   poll.registry().register(&mut source_fd, token, Interest::READABLE | Interest::WRITABLE)
   ```

2. **使用 SourceFd 包裝 RawFd**:
   ```rust
   use mio::unix::SourceFd;
   let mut source_fd = SourceFd(&fd);
   ```

## 測試

```bash
# 運行所有測試
cargo test -p xfw-platform

# 運行特定測試文件
cargo test -p xfw-platform --test buffer_tests
cargo test -p xfw-platform --test surface_tests
cargo test -p xfw-platform --test xdg_tests
cargo test -p xfw-platform --test event_loop_tests
cargo test -p xfw-platform --test input_tests
```