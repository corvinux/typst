use rex::error::{Error, LayoutError};
use rex::font::FontContext;
use rex::layout::{LayoutSettings, Style};
use rex::parser::color::RGBA;
use rex::render::{Backend, Cursor, Renderer};
use typst::font::Font;

use crate::prelude::*;
use crate::text::{families, variant, LinebreakNode, SpaceNode, TextNode};

/// Turn a math node into TeX math code.
#[capability]
pub trait Texify {
    /// Perform the conversion.
    fn texify(&self) -> EcoString;
}

impl Texify for Content {
    fn texify(&self) -> EcoString {
        if self.is::<SpaceNode>() {
            return EcoString::new();
        }

        if self.is::<LinebreakNode>() {
            return r"\\".into();
        }

        if let Some(node) = self.with::<dyn Texify>() {
            return node.texify();
        }

        panic!("{self:?} is not math");
    }
}

/// Layout a TeX formula into a frame.
pub fn layout_tex(
    vt: &Vt,
    tex: &str,
    display: bool,
    styles: StyleChain,
) -> SourceResult<Fragment> {
    // Load the font.
    let variant = variant(styles);
    let world = vt.world();
    let mut font = None;
    for family in families(styles) {
        font = world.book().select(family, variant).and_then(|id| world.font(id));
        if font.as_ref().map_or(false, |font| font.math().is_some()) {
            break;
        }
    }

    // Prepare the font context.
    let font = font.expect("failed to find suitable math font");
    let ctx = font
        .math()
        .map(|math| FontContext::new(font.ttf(), math))
        .expect("failed to create font context");

    // Layout the formula.
    let em = styles.get(TextNode::SIZE);
    let style = if display { Style::Display } else { Style::Text };
    let settings = LayoutSettings::new(&ctx, em.to_pt(), style);
    let renderer = Renderer::new();
    let layout = renderer
        .layout(&tex, settings)
        .map_err(|err| match err {
            Error::Parse(err) => err.to_string(),
            Error::Layout(LayoutError::Font(err)) => err.to_string(),
        })
        .expect("failed to layout with rex");

    // Determine the metrics.
    let (x0, y0, x1, y1) = renderer.size(&layout);
    let width = Abs::pt(x1 - x0);
    let mut top = Abs::pt(y1);
    let mut bottom = Abs::pt(-y0);
    if style != Style::Display {
        let metrics = font.metrics();
        top = styles.get(TextNode::TOP_EDGE).resolve(styles, metrics);
        bottom = -styles.get(TextNode::BOTTOM_EDGE).resolve(styles, metrics);
    };

    // Prepare a frame rendering backend.
    let size = Size::new(width, top + bottom);
    let mut backend = FrameBackend {
        frame: {
            let mut frame = Frame::new(size);
            frame.set_baseline(top);
            frame
        },
        baseline: top,
        font: font.clone(),
        fill: styles.get(TextNode::FILL),
        lang: styles.get(TextNode::LANG),
        colors: vec![],
    };

    // Render into the frame.
    renderer.render(&layout, &mut backend);

    Ok(Fragment::frame(backend.frame))
}

/// A ReX rendering backend that renders into a frame.
struct FrameBackend {
    frame: Frame,
    baseline: Abs,
    font: Font,
    fill: Paint,
    lang: Lang,
    colors: Vec<RGBA>,
}

impl FrameBackend {
    /// The currently active fill paint.
    fn fill(&self) -> Paint {
        self.colors
            .last()
            .map(|&RGBA(r, g, b, a)| RgbaColor::new(r, g, b, a).into())
            .unwrap_or(self.fill)
    }

    /// Convert a cursor to a point.
    fn transform(&self, cursor: Cursor) -> Point {
        Point::new(Abs::pt(cursor.x), self.baseline + Abs::pt(cursor.y))
    }
}

impl Backend for FrameBackend {
    fn symbol(&mut self, pos: Cursor, gid: u16, scale: f64) {
        self.frame.push(
            self.transform(pos),
            Element::Text(Text {
                font: self.font.clone(),
                size: Abs::pt(scale),
                fill: self.fill(),
                lang: self.lang,
                glyphs: vec![Glyph {
                    id: gid,
                    x_advance: Em::new(0.0),
                    x_offset: Em::new(0.0),
                    c: ' ',
                }],
            }),
        );
    }

    fn rule(&mut self, pos: Cursor, width: f64, height: f64) {
        self.frame.push(
            self.transform(pos),
            Element::Shape(Shape {
                geometry: Geometry::Rect(Size::new(Abs::pt(width), Abs::pt(height))),
                fill: Some(self.fill()),
                stroke: None,
            }),
        );
    }

    fn begin_color(&mut self, color: RGBA) {
        self.colors.push(color);
    }

    fn end_color(&mut self) {
        self.colors.pop();
    }
}