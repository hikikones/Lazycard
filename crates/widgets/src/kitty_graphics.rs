use ratatui::{buffer::Buffer, layout::Rect};

// TODO: Handle division by zero?
// Use Size struct from ratatui insted of custom Area.

pub struct KittyGraphics {
    frames: Vec<image::Frame>,
    zlib_deflate: ZlibDeflate,
    base64: Base64,
    formatter: utils::Formatter,
    cell_size: CellSize,
    kitty_verbosity: KittyVerbosity,
}

impl KittyGraphics {
    pub const fn new(cell_size: CellSize) -> Self {
        Self {
            frames: Vec::new(),
            zlib_deflate: ZlibDeflate::new(),
            base64: Base64::new(),
            formatter: utils::Formatter::new(),
            cell_size,
            kitty_verbosity: KittyVerbosity::Silent,
        }
    }

    pub fn load(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), image::error::ImageError> {
        use image::AnimationDecoder;

        let path = path.as_ref();
        let reader = image::ImageReader::open(path)?.with_guessed_format()?;

        let Some(image_format) = reader.format() else {
            let image = reader.decode()?.to_rgba8();
            self.frames.push(image::Frame::new(image));
            return Ok(());
        };

        self.frames.clear();

        match image_format {
            image::ImageFormat::Png => {
                let png_decoder = image::codecs::png::PngDecoder::new(reader.into_inner())?;
                if png_decoder.is_apng()? {
                    for frame_res in png_decoder.apng()?.into_frames() {
                        match frame_res {
                            Ok(frame) => self.frames.push(frame),
                            Err(err) => {
                                self.frames.clear();
                                return Err(err);
                            }
                        }
                    }
                } else {
                    let mut reader = image::ImageReader::open(path).unwrap();
                    reader.set_format(image::ImageFormat::Png);
                    let image = reader.decode()?.to_rgba8();
                    self.frames.push(image::Frame::new(image));
                }
            }
            image::ImageFormat::Gif => {
                for frame_res in
                    image::codecs::gif::GifDecoder::new(reader.into_inner())?.into_frames()
                {
                    match frame_res {
                        Ok(frame) => self.frames.push(frame),
                        Err(err) => {
                            self.frames.clear();
                            return Err(err);
                        }
                    }
                }
            }
            image::ImageFormat::WebP => {
                for frame_res in
                    image::codecs::webp::WebPDecoder::new(reader.into_inner())?.into_frames()
                {
                    match frame_res {
                        Ok(frame) => self.frames.push(frame),
                        Err(err) => {
                            self.frames.clear();
                            return Err(err);
                        }
                    }
                }
            }
            _ => {
                let image = reader.decode()?.to_rgba8();
                self.frames.push(image::Frame::new(image));
            }
        }

        Ok(())
    }

    pub fn encode(&mut self, id: u32) -> std::io::Result<Dimensions> {
        use std::io::Write;

        debug_assert_ne!(id, 0);

        let rgba = self.frames[0].buffer();
        let dims = Dimensions::from_tuple(rgba.dimensions());
        let compressed = self.zlib_deflate.compress(rgba.as_raw()).unwrap();
        let b64 = self.base64.encode(compressed);

        let mut stdout = std::io::stdout().lock();

        // Encode root image
        let (root_header, chunk_header) = self.formatter.push_fmt2(
            format_args!(
                "{},{},{},{},{},{}",
                KittyAction::Transmit,
                KittyImageFormat::Rgba32(dims),
                KittyTransfer::Direct,
                KittyId(id),
                KittyCompression::ZlibDeflate,
                self.kitty_verbosity
            ),
            format_args!("{},{}", KittyId(id), self.kitty_verbosity),
        );
        Self::write_chunks(
            self.formatter.slice(root_header),
            self.formatter.slice(chunk_header),
            b64,
            &mut stdout,
        )?;
        self.formatter.clear();

        // Animated image
        if self.frames.len() > 1 {
            for i in 1..self.frames.len() {
                let delay = self.frames[i].delay().numer_denom_ms().0 as i32;
                let rgba = self.frames[i].buffer();
                let dims = Dimensions::from_tuple(rgba.dimensions());
                let compressed = self.zlib_deflate.compress(rgba.as_raw()).unwrap();
                let b64 = self.base64.encode(compressed);

                let (root_header, chunk_header) = self.formatter.push_fmt2(
                    format_args!(
                        "{},{},{},{},{},{},{}",
                        KittyAction::AnimationTransmitFrame,
                        KittyImageFormat::Rgba32(dims),
                        KittyTransfer::Direct,
                        KittyId(id),
                        KittyAnimationGap(delay),
                        KittyCompression::ZlibDeflate,
                        self.kitty_verbosity
                    ),
                    format_args!(
                        "{},{},{}",
                        KittyAction::AnimationTransmitFrame,
                        KittyId(id),
                        self.kitty_verbosity
                    ),
                );
                Self::write_chunks(
                    self.formatter.slice(root_header),
                    self.formatter.slice(chunk_header),
                    b64,
                    &mut stdout,
                )?;
                self.formatter.clear();
            }

            // Set animation controls
            write!(
                stdout,
                "\x1b_G{},{},{},{},{}\x1b\\",
                KittyAction::AnimationControl,
                KittyId(id),
                KittyAnimationState::RunNormal,
                KittyAnimationLoop::Forever,
                self.kitty_verbosity
            )?;
        }

        stdout.flush()?;

        Ok(dims)
    }

    pub fn render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        id: u32,
        dims: Dimensions,
        resize: ResizeMode,
        alignment: crate::utils::Alignment,
    ) {
        let area = area.intersection(buf.area);

        if area.is_empty() {
            return;
        }

        // Start kitty graphics
        self.formatter.push_fmt(format_args!(
            "\x1b_G{},{},{},{},{}",
            KittyAction::Display,
            KittyId(id),
            KittyPlacement(id),
            KittyCursorMovement::NoMovement,
            self.kitty_verbosity
        ));

        // Modify image layout if necessary
        let image_size = match resize {
            ResizeMode::None => self.area(dims),
            ResizeMode::Fit => {
                let max_dims = self.dimensions(area);
                let width_ratio = dims.width as f64 / max_dims.width as f64;
                let height_ratio = dims.height as f64 / max_dims.height as f64;

                // Scale by either columns or rows to maintain aspect ratio
                let scaling = if width_ratio >= height_ratio {
                    KittyScale::Columns(area.width.min(self.columns(dims.width)))
                } else {
                    KittyScale::Rows(area.height.min(self.rows(dims.height)))
                };
                self.formatter.push_fmt(format_args!(",{}", scaling));
                self.area(Self::resize(dims, max_dims))
            }
            ResizeMode::Stretch => {
                self.formatter.push_fmt(format_args!(
                    ",{}",
                    KittyScale::Stretch(area.width, area.height)
                ));
                Area::from_rect(area)
            }
            ResizeMode::FitWidthCropHeight { rows_outside_top } => {
                let max_width = self.width(area.width);
                let resized_dims = Self::resize(dims, dims.with_width(max_width));
                let resized_area = self.area(resized_dims);

                if resized_area.rows > area.height {
                    // Need to crop height with source rectangle x, y, width, height
                    let y = self.height(rows_outside_top);
                    let height = self.height(area.height);
                    let h_ratio = dims.height as f64 / resized_dims.height as f64;
                    self.formatter.push_fmt(format_args!(
                        ",{},{}",
                        KittyScale::Columns(area.width.min(resized_area.columns)),
                        KittyCrop {
                            x: 0,
                            y: (y as f64 * h_ratio).round() as u32,
                            width: dims.width,
                            height: (height as f64 * h_ratio).round() as u32
                        }
                    ));
                    resized_area.with_rows(area.height)
                } else {
                    // No vertical cropping needed, just scale by width
                    self.formatter.push_fmt(format_args!(
                        ",{}",
                        KittyScale::Columns(area.width.min(self.columns(dims.width))),
                    ));
                    resized_area.with_rows(self.rows(dims.height))
                }
            }
        };

        // End kitty graphics
        self.formatter.push_str("\x1b\\");

        // Image alignment
        let Rect { x, y, .. } = crate::utils::align(
            Rect {
                width: image_size.columns,
                height: image_size.rows,
                ..area
            },
            area,
            alignment,
        );

        // Time to render by writing to stdout
        {
            use std::io::Write;
            let mut stdout = std::io::stdout().lock();

            // Set cursor position (row, col)
            write!(stdout, "\x1b[{};{}H", y + 1, x + 1).unwrap();

            // Transmit kitty display command
            write!(stdout, "{}", self.formatter).unwrap();

            // Make sure all is written
            stdout.flush().unwrap();
        }

        self.formatter.clear();
    }

    pub fn delete_id(&self, id: u32) -> std::io::Result<()> {
        use std::io::Write;

        let mut stdout = std::io::stdout();
        write!(
            stdout,
            "\x1b_G{},{},{}\x1b\\",
            KittyAction::Delete(KittyDelete::Id),
            KittyId(id),
            self.kitty_verbosity
        )?;
        stdout.flush()?;

        Ok(())
    }

    pub fn delete_ids<I>(&self, ids: I) -> std::io::Result<()>
    where
        I: IntoIterator<Item = u32>,
        I::IntoIter: ExactSizeIterator,
    {
        use std::io::Write;

        let ids = ids.into_iter();
        if ids.len() == 0 {
            return Ok(());
        }

        let mut stdout = std::io::stdout().lock();
        for id in ids {
            write!(
                stdout,
                "\x1b_G{},{},{}\x1b\\",
                KittyAction::Delete(KittyDelete::Id),
                KittyId(id),
                self.kitty_verbosity
            )?;
        }
        stdout.flush()?;

        Ok(())
    }

    pub fn delete_range(&self, min_inclusive: u32, max_inclusive: u32) -> std::io::Result<()> {
        use std::io::Write;

        debug_assert!(min_inclusive <= max_inclusive);

        let mut stdout = std::io::stdout();
        write!(
            stdout,
            "\x1b_G{},{}\x1b\\",
            KittyAction::Delete(KittyDelete::Range(min_inclusive, max_inclusive)),
            self.kitty_verbosity
        )?;
        stdout.flush()?;

        Ok(())
    }

    pub fn delete_all(&self) -> std::io::Result<()> {
        use std::io::Write;

        let mut stdout = std::io::stdout();
        write!(
            stdout,
            "\x1b_G{},{}\x1b\\",
            KittyAction::Delete(KittyDelete::AllVisible),
            self.kitty_verbosity
        )?;
        stdout.flush()?;

        Ok(())
    }

    pub const fn width(&self, columns: u16) -> u32 {
        self.cell_size.width(columns)
    }

    pub const fn height(&self, rows: u16) -> u32 {
        self.cell_size.height(rows)
    }

    pub const fn columns(&self, width: u32) -> u16 {
        self.cell_size.columns(width)
    }

    pub const fn rows(&self, height: u32) -> u16 {
        self.cell_size.rows(height)
    }

    pub const fn dimensions(&self, area: Rect) -> Dimensions {
        self.cell_size.dimensions(area)
    }

    pub const fn area(&self, dims: Dimensions) -> Area {
        self.cell_size.area(dims)
    }

    pub fn resize(dims: Dimensions, max: Dimensions) -> Dimensions {
        let (rw, rh) =
            utils::resize_dimensions(dims.width, dims.height, max.width, max.height, false);
        Dimensions {
            width: rw,
            height: rh,
        }
    }

    fn write_chunks(
        root_header: &str,
        chunk_header: &str,
        b64: &str,
        w: &mut impl std::io::Write,
    ) -> std::io::Result<()> {
        const CHUNK_SIZE: usize = 4096;
        let b64_len = b64.len();

        if b64_len <= CHUNK_SIZE {
            return write!(w, "\x1b_G{root_header};{b64}\x1b\\");
        }

        write!(w, "\x1b_G{root_header},m=1;{}\x1b\\", &b64[0..CHUNK_SIZE])?;
        let mut start = CHUNK_SIZE;
        let mut end = CHUNK_SIZE * 2;
        while end < b64_len {
            write!(w, "\x1b_G{chunk_header},m=1;{}\x1b\\", &b64[start..end])?;
            start = end;
            end += CHUNK_SIZE;
        }
        write!(w, "\x1b_G{chunk_header},m=0;{}\x1b\\", &b64[start..])?;

        Ok(())
    }
}

/// The pixel dimensions of a single cell in the terminal.
#[derive(Debug, Clone, Copy)]
pub struct CellSize {
    pub width: u32,
    pub height: u32,
}

impl CellSize {
    pub const DEFAULT: Self = Self {
        width: 10,
        height: 20,
    };

    pub fn query() -> std::io::Result<Option<Self>> {
        use std::io::{Read, Write};

        // TODO: For more terminal support, you can run two other CSI t escape codes:
        //  1. b"\x1b[14t" returns the screen size in pixels (height, width), and
        //  2. b"\x1b[18t" returns the screen size in character cells (rows, cols).
        // From this, you can calculate the pixel size.

        ratatui::crossterm::terminal::enable_raw_mode()?;

        // Send two escape codes at once.
        // First, the "[16t" for pixel dimensions of a single cell.
        // Not many terminals support this, but this is fine as they probably
        // don't support kitty graphics either.
        // Second, the "[5n" to ensure a response in case the first one is not supported.
        // The second one is a Device Status Report that all terminals implement.
        // If no response is sent back, then the stdin read will block forever.
        let mut stdout = std::io::stdout();
        stdout.write_all(b"\x1b[16t\x1b[5n")?;
        stdout.flush()?;

        // Get the response from stdin
        let mut buffer = [0; 64];
        let n = std::io::stdin().read(&mut buffer)?;

        ratatui::crossterm::terminal::disable_raw_mode()?;

        // Parse the response which will look like "\u{1b}[6;<HEIGHT>;<WIDTH>t\u{1b}[0n".
        // If no response for pixel size, then only last part will be available which we can ignore.
        let s = String::from_utf8_lossy(&buffer[..n]);
        let mut split = s.split(";");
        split.next();
        let height = split.next().map(|h| h.parse::<u32>());
        let width = split
            .next()
            .map(|w| w.find('t').map(|i| w[..i].parse::<u32>()))
            .flatten();

        match (height, width) {
            (Some(Ok(height)), Some(Ok(width))) => Ok(Some(Self { width, height })),
            _ => Ok(None),
        }
    }

    pub const fn width(&self, columns: u16) -> u32 {
        columns as u32 * self.width
    }

    pub const fn height(&self, rows: u16) -> u32 {
        rows as u32 * self.height
    }

    pub const fn columns(&self, width: u32) -> u16 {
        width.div_ceil(self.width) as u16
    }

    pub const fn rows(&self, height: u32) -> u16 {
        height.div_ceil(self.height) as u16
    }

    pub const fn dimensions(&self, area: Rect) -> Dimensions {
        Dimensions {
            width: self.width(area.width),
            height: self.height(area.height),
        }
    }

    pub const fn area(&self, dims: Dimensions) -> Area {
        Area {
            columns: self.columns(dims.width),
            rows: self.rows(dims.height),
        }
    }
}

impl Default for CellSize {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ResizeMode {
    None,
    #[default]
    Fit,
    Stretch,
    FitWidthCropHeight {
        rows_outside_top: u16,
    },
}

/// The area for an image.
#[derive(Debug, Clone, Copy)]
pub struct Area {
    pub columns: u16,
    pub rows: u16,
}

impl Area {
    pub const fn new(columns: u16, rows: u16) -> Self {
        Self { columns, rows }
    }

    pub const fn from_rect(area: Rect) -> Self {
        Self::new(area.width, area.height)
    }

    pub const fn with_columns(mut self, columns: u16) -> Self {
        self.columns = columns;
        self
    }

    pub const fn with_rows(mut self, rows: u16) -> Self {
        self.rows = rows;
        self
    }
}

/// The pixel dimensions for an image.
#[derive(Debug, Clone, Copy)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Dimensions {
    pub const fn from_tuple(wh: (u32, u32)) -> Self {
        Self {
            width: wh.0,
            height: wh.1,
        }
    }

    pub const fn with_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    pub const fn with_height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }
}

#[derive(Debug, Clone, Copy)]
enum KittyImageFormat {
    // Rgb24(Dimensions),
    Rgba32(Dimensions),
    // Png,
}

impl std::fmt::Display for KittyImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rgba32(dims) => {
                f.write_fmt(format_args!("f=32,s={},v={}", dims.width, dims.height))
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum KittyTransfer {
    Direct,
    // File,
    // TempFile,
    // SharedMemory
}

impl std::fmt::Display for KittyTransfer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KittyTransfer::Direct => f.write_str("t=d"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum KittyAction {
    Transmit,
    // TransmitAndDisplay,
    // QueryTerminal,
    Display,
    Delete(KittyDelete),
    AnimationTransmitFrame,
    // AnimationComposeFrame,
    AnimationControl,
}

impl std::fmt::Display for KittyAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Transmit => f.write_str("a=t"),
            Self::Display => f.write_str("a=p"),
            Self::Delete(delete) => f.write_fmt(format_args!("a=d,{}", delete)),
            Self::AnimationTransmitFrame => f.write_str("a=f"),
            Self::AnimationControl => f.write_str("a=a"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct KittyId(u32);

impl std::fmt::Display for KittyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("i={}", self.0))
    }
}

#[derive(Debug, Clone, Copy)]
struct KittyPlacement(u32);

impl std::fmt::Display for KittyPlacement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("p={}", self.0))
    }
}

#[derive(Debug, Clone, Copy)]
enum KittyScale {
    Columns(u16),
    Rows(u16),
    Stretch(u16, u16),
}

impl std::fmt::Display for KittyScale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Columns(cols) => f.write_fmt(format_args!("c={}", cols)),
            Self::Rows(rows) => f.write_fmt(format_args!("r={}", rows)),
            Self::Stretch(cols, rows) => f.write_fmt(format_args!("c={},r={}", cols, rows)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct KittyCrop {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl std::fmt::Display for KittyCrop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            x,
            y,
            width,
            height,
        } = *self;
        f.write_fmt(format_args!("x={},y={},w={},h={}", x, y, width, height))
    }
}

#[derive(Debug, Clone, Copy)]
enum KittyDelete {
    AllVisible,
    Id,
    Range(u32, u32),
}

impl std::fmt::Display for KittyDelete {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::AllVisible => f.write_str("d=a"),
            Self::Id => f.write_str("d=i"),
            Self::Range(min, max) => f.write_fmt(format_args!("d=r,x={},y={}", min, max)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum KittyCompression {
    ZlibDeflate,
}

impl std::fmt::Display for KittyCompression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZlibDeflate => f.write_str("o=z"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum KittyCursorMovement {
    // MoveToAfterImage = 0,
    NoMovement = 1,
}

impl std::fmt::Display for KittyCursorMovement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("C={}", *self as u8))
    }
}

#[derive(Debug, Clone, Copy)]
struct KittyAnimationGap(i32);

impl std::fmt::Display for KittyAnimationGap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("z={}", self.0))
    }
}

#[derive(Debug, Clone, Copy)]
enum KittyAnimationState {
    // Stop = 1,
    // RunWaitLoad = 2,
    RunNormal = 3,
}

impl std::fmt::Display for KittyAnimationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("s={}", *self as u8))
    }
}

#[derive(Debug, Clone, Copy)]
enum KittyAnimationLoop {
    // Ignore,
    Forever,
    // Amount(u32),
}

impl std::fmt::Display for KittyAnimationLoop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            // Self::Ignore => f.write_str("v=0"),
            Self::Forever => f.write_str("v=1"),
            // Self::Amount(n) => f.write_fmt(format_args!("v={}", n + 1)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum KittyVerbosity {
    // All = 0,
    // ErrorsOnly = 1,
    Silent = 2,
}

impl std::fmt::Display for KittyVerbosity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("q={}", *self as u8))
    }
}

struct ZlibDeflate(Vec<u8>);

impl ZlibDeflate {
    const fn new() -> Self {
        Self(Vec::new())
    }

    fn compress(&mut self, input: &[u8]) -> Result<&[u8], zlib_rs::ReturnCode> {
        // Reset buffer with zeroes
        let zeroes = zlib_rs::compress_bound(input.len());
        self.0.clear();
        self.0.extend(std::iter::repeat_n(0, zeroes));

        // Compress
        let config = zlib_rs::DeflateConfig::default();
        let (compressed, rc) = zlib_rs::compress_slice(&mut self.0, input, config);
        match rc {
            zlib_rs::ReturnCode::Ok => Ok(compressed),
            _ => Err(rc),
        }
    }
}

struct Base64(String);

impl Base64 {
    const fn new() -> Self {
        Self(String::new())
    }

    fn encode(&mut self, input: &[u8]) -> &str {
        use base64::{Engine, engine::general_purpose::STANDARD};

        self.0.clear();
        STANDARD.encode_string(input, &mut self.0);
        self.0.as_str()
    }
}
