use std::sync::RwLock;
use std::sync::LazyLock;

use ratatui::style::Color;

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub id: &'static str,
    pub name: &'static str,
    pub base: Color,
    pub surface: Color,
    pub overlay: Color,
    pub muted: Color,
    pub subtle: Color,
    pub text: Color,
    pub love: Color,
    pub gold: Color,
    pub rose: Color,
    pub pine: Color,
    pub foam: Color,
    pub iris: Color,
    pub hl_med: Color,
    pub hl_high: Color,

    pub background: Color,
    pub background_panel: Color,
    pub background_element: Color,
    pub background_menu: Color,
    pub text_muted: Color,
    pub border: Color,
    pub border_active: Color,
    pub primary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
}

pub const fn rose_pine() -> Theme {
    let base = Color::Rgb(25, 23, 36);
    let surface = Color::Rgb(31, 29, 46);
    let overlay = Color::Rgb(38, 35, 58);
    let muted = Color::Rgb(110, 106, 134);
    let subtle = Color::Rgb(144, 140, 170);
    let text = Color::Rgb(224, 222, 244);
    let love = Color::Rgb(235, 111, 146);
    let gold = Color::Rgb(246, 193, 119);
    let rose = Color::Rgb(235, 188, 186);
    let pine = Color::Rgb(49, 116, 143);
    let foam = Color::Rgb(156, 207, 216);
    let iris = Color::Rgb(196, 167, 231);
    let hl_med = Color::Rgb(64, 61, 82);
    let hl_high = Color::Rgb(82, 79, 103);
    Theme {
        id: "rose-pine", name: "Rosé Pine",
        base, surface, overlay, muted, subtle, text,
        love, gold, rose, pine, foam, iris, hl_med, hl_high,
        background: base, background_panel: surface, background_element: overlay,
        background_menu: hl_med, text_muted: muted, border: hl_med, border_active: hl_high,
        primary: iris, accent: foam, success: pine, warning: gold, error: love,
    }
}

pub const fn catppuccin_mocha() -> Theme {
    let base = Color::Rgb(30, 30, 46);
    let surface = Color::Rgb(49, 50, 68);
    let overlay = Color::Rgb(69, 71, 90);
    let muted = Color::Rgb(127, 132, 156);
    let subtle = Color::Rgb(166, 173, 200);
    let text = Color::Rgb(205, 214, 244);
    let love = Color::Rgb(243, 139, 168);
    let gold = Color::Rgb(249, 226, 175);
    let rose = Color::Rgb(245, 194, 231);
    let pine = Color::Rgb(166, 227, 161);
    let foam = Color::Rgb(137, 220, 235);
    let iris = Color::Rgb(203, 166, 247);
    let hl_med = Color::Rgb(88, 91, 112);
    let hl_high = Color::Rgb(108, 112, 134);
    Theme {
        id: "catppuccin-mocha", name: "Catppuccin Mocha",
        base, surface, overlay, muted, subtle, text,
        love, gold, rose, pine, foam, iris, hl_med, hl_high,
        background: base, background_panel: surface, background_element: overlay,
        background_menu: hl_med, text_muted: muted, border: hl_med, border_active: hl_high,
        primary: iris, accent: foam, success: pine, warning: gold, error: love,
    }
}

pub const fn tokyo_night() -> Theme {
    let base = Color::Rgb(26, 27, 38);
    let surface = Color::Rgb(36, 40, 59);
    let overlay = Color::Rgb(52, 59, 88);
    let muted = Color::Rgb(86, 95, 137);
    let subtle = Color::Rgb(154, 165, 206);
    let text = Color::Rgb(192, 202, 245);
    let love = Color::Rgb(247, 118, 142);
    let gold = Color::Rgb(224, 175, 104);
    let rose = Color::Rgb(255, 158, 100);
    let pine = Color::Rgb(158, 206, 106);
    let foam = Color::Rgb(125, 207, 255);
    let iris = Color::Rgb(187, 154, 247);
    let hl_med = Color::Rgb(65, 72, 104);
    let hl_high = Color::Rgb(86, 95, 137);
    Theme {
        id: "tokyo-night", name: "Tokyo Night",
        base, surface, overlay, muted, subtle, text,
        love, gold, rose, pine, foam, iris, hl_med, hl_high,
        background: base, background_panel: surface, background_element: overlay,
        background_menu: hl_med, text_muted: muted, border: hl_med, border_active: hl_high,
        primary: iris, accent: foam, success: pine, warning: gold, error: love,
    }
}

pub const fn gruvbox_dark() -> Theme {
    let base = Color::Rgb(40, 40, 40);
    let surface = Color::Rgb(50, 48, 47);
    let overlay = Color::Rgb(60, 56, 54);
    let muted = Color::Rgb(124, 111, 100);
    let subtle = Color::Rgb(168, 153, 132);
    let text = Color::Rgb(235, 219, 178);
    let love = Color::Rgb(251, 73, 52);
    let gold = Color::Rgb(250, 189, 47);
    let rose = Color::Rgb(214, 93, 14);
    let pine = Color::Rgb(184, 187, 38);
    let foam = Color::Rgb(131, 165, 152);
    let iris = Color::Rgb(211, 134, 155);
    let hl_med = Color::Rgb(80, 73, 69);
    let hl_high = Color::Rgb(102, 92, 84);
    Theme {
        id: "gruvbox-dark", name: "Gruvbox Dark",
        base, surface, overlay, muted, subtle, text,
        love, gold, rose, pine, foam, iris, hl_med, hl_high,
        background: base, background_panel: surface, background_element: overlay,
        background_menu: hl_med, text_muted: muted, border: hl_med, border_active: hl_high,
        primary: iris, accent: foam, success: pine, warning: gold, error: love,
    }
}

pub const THEMES: &[Theme] = &[
    rose_pine(),
    catppuccin_mocha(),
    tokyo_night(),
    gruvbox_dark(),
];

static CURRENT: LazyLock<RwLock<Theme>> = LazyLock::new(|| RwLock::new(rose_pine()));

pub fn current() -> Theme { *CURRENT.read().unwrap() }

pub fn set_theme(id: &str) -> bool {
    for t in THEMES {
        if t.id == id {
            *CURRENT.write().unwrap() = *t;
            return true;
        }
    }
    false
}

// Token accessors — preserve the existing call-site shape (`theme::FOO`) but
// dispatch to the runtime palette.
#[allow(non_snake_case)] pub fn BASE()    -> Color { current().base }
#[allow(non_snake_case)] pub fn SURFACE() -> Color { current().surface }
#[allow(non_snake_case)] pub fn OVERLAY() -> Color { current().overlay }
#[allow(non_snake_case)] pub fn MUTED()   -> Color { current().muted }
#[allow(non_snake_case)] pub fn SUBTLE()  -> Color { current().subtle }
#[allow(non_snake_case)] pub fn TEXT()    -> Color { current().text }
#[allow(non_snake_case)] pub fn LOVE()    -> Color { current().love }
#[allow(non_snake_case)] pub fn GOLD()    -> Color { current().gold }
#[allow(non_snake_case)] pub fn ROSE()    -> Color { current().rose }
#[allow(non_snake_case)] pub fn PINE()    -> Color { current().pine }
#[allow(non_snake_case)] pub fn FOAM()    -> Color { current().foam }
#[allow(non_snake_case)] pub fn IRIS()    -> Color { current().iris }
#[allow(non_snake_case)] pub fn HL_MED()  -> Color { current().hl_med }
#[allow(non_snake_case)] pub fn HL_HIGH() -> Color { current().hl_high }

#[allow(non_snake_case)] pub fn BACKGROUND()         -> Color { current().background }
#[allow(non_snake_case)] pub fn BACKGROUND_PANEL()   -> Color { current().background_panel }
#[allow(non_snake_case)] pub fn BACKGROUND_ELEMENT() -> Color { current().background_element }
#[allow(non_snake_case)] pub fn BACKGROUND_MENU()    -> Color { current().background_menu }
#[allow(non_snake_case)] pub fn TEXT_MUTED()         -> Color { current().text_muted }
#[allow(non_snake_case)] pub fn BORDER()             -> Color { current().border }
#[allow(non_snake_case)] pub fn BORDER_ACTIVE()      -> Color { current().border_active }
#[allow(non_snake_case)] pub fn PRIMARY()            -> Color { current().primary }
#[allow(non_snake_case)] pub fn ACCENT()             -> Color { current().accent }
#[allow(non_snake_case)] pub fn SUCCESS()            -> Color { current().success }
#[allow(non_snake_case)] pub fn WARNING()            -> Color { current().warning }
#[allow(non_snake_case)] pub fn ERROR()              -> Color { current().error }
