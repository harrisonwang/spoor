#![allow(dead_code)]

use crate::parse::pdf_media::PageImage;

/// Internal PDF layout representation.
///
/// This is intentionally not part of the public API yet.  The first milestone
/// keeps the existing page-oriented text output stable while giving future PDF
/// work (span extraction, reading order, header/footer classification, links,
/// and chunk metadata) a single place to attach structure and diagnostics.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfLayoutDocument {
    pub(crate) pages: Vec<PdfLayoutPage>,
    pub(crate) diagnostics: Vec<PdfLayoutDiagnostic>,
}

impl PdfLayoutDocument {
    pub(crate) fn from_page_text_and_images(
        pages: Vec<String>,
        images: Vec<Vec<PageImage>>,
    ) -> Self {
        let numbered_pages = pages
            .into_iter()
            .enumerate()
            .map(|(index, text)| (index + 1, text));
        Self::from_numbered_page_text_and_images(numbered_pages, images)
    }

    pub(crate) fn from_numbered_page_text_and_images(
        pages: impl IntoIterator<Item = (usize, String)>,
        images: Vec<Vec<PageImage>>,
    ) -> Self {
        let pages = pages
            .into_iter()
            .enumerate()
            .map(|(image_index, (number, text))| {
                let page_images = images.get(image_index).cloned().unwrap_or_default();
                PdfLayoutPage::from_page_text(number, text, page_images)
            })
            .collect();

        Self {
            pages,
            diagnostics: Vec::new(),
        }
    }

    pub(crate) fn has_text(&self) -> bool {
        self.pages.iter().any(PdfLayoutPage::has_text)
    }

    pub(crate) fn has_images(&self) -> bool {
        self.pages.iter().any(|page| !page.images.is_empty())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfLayoutPage {
    pub(crate) number: usize,
    pub(crate) width: Option<f32>,
    pub(crate) height: Option<f32>,
    pub(crate) spans: Vec<PdfTextSpan>,
    pub(crate) lines: Vec<PdfTextLine>,
    pub(crate) blocks: Vec<PdfBlock>,
    pub(crate) images: Vec<PdfImageObject>,
    pub(crate) links: Vec<PdfLinkAnnotation>,
    pub(crate) diagnostics: Vec<PdfLayoutDiagnostic>,
}

impl PdfLayoutPage {
    fn from_page_text(number: usize, text: String, images: Vec<PageImage>) -> Self {
        let mut blocks = Vec::new();
        if !text.is_empty() {
            blocks.push(PdfBlock {
                kind: PdfBlockKind::Paragraph,
                text,
                bbox: None,
                reading_order: 0,
                confidence: 1.0,
            });
        }

        let images = images
            .into_iter()
            .enumerate()
            .map(|(index, image)| PdfImageObject {
                id: image.id,
                generation: image.generation,
                page_image_number: index + 1,
                bbox: None,
                extractable: image.extractable,
            })
            .collect();

        Self {
            number,
            width: None,
            height: None,
            spans: Vec::new(),
            lines: Vec::new(),
            blocks,
            images,
            links: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub(crate) fn text(&self) -> &str {
        self.blocks
            .iter()
            .find(|block| matches!(block.kind, PdfBlockKind::Paragraph))
            .map(|block| block.text.as_str())
            .unwrap_or("")
    }

    pub(crate) fn has_text(&self) -> bool {
        !self.text().trim().is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfTextSpan {
    pub(crate) text: String,
    pub(crate) bbox: Option<PdfRect>,
    pub(crate) font_size: Option<f32>,
    pub(crate) font_name: Option<String>,
    pub(crate) font_flags: PdfFontFlags,
    pub(crate) source_order: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfTextLine {
    pub(crate) spans: Vec<usize>,
    pub(crate) bbox: Option<PdfRect>,
    pub(crate) baseline_y: Option<f32>,
    pub(crate) column_hint: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfBlock {
    pub(crate) kind: PdfBlockKind,
    pub(crate) text: String,
    pub(crate) bbox: Option<PdfRect>,
    pub(crate) reading_order: usize,
    pub(crate) confidence: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PdfBlockKind {
    Paragraph,
    Heading,
    ListItem,
    HeaderFooter,
    Watermark,
    Caption,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfImageObject {
    pub(crate) id: u32,
    pub(crate) generation: u16,
    pub(crate) page_image_number: usize,
    pub(crate) bbox: Option<PdfRect>,
    pub(crate) extractable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfLinkAnnotation {
    pub(crate) target: String,
    pub(crate) bbox: Option<PdfRect>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfLayoutDiagnostic {
    pub(crate) kind: PdfLayoutDiagnosticKind,
    pub(crate) page_number: Option<usize>,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PdfLayoutDiagnosticKind {
    NoTextLayer,
    SuspiciousTextLayer,
    ImageOmitted,
    ReadingOrderInferred,
    ReadingOrderUncertain,
    HeaderFooterClassified,
    HeadingInferred,
    LinkUnassociated,
    LayoutExtractionFailed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PdfRect {
    pub(crate) x0: f32,
    pub(crate) y0: f32,
    pub(crate) x1: f32,
    pub(crate) y1: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct PdfFontFlags {
    pub(crate) bold: bool,
    pub(crate) italic: bool,
    pub(crate) monospace: bool,
}

#[cfg(test)]
mod tests {
    use super::{PdfBlockKind, PdfLayoutDocument};
    use crate::parse::pdf_media::PageImage;

    #[test]
    fn page_text_and_images_seed_layout_skeleton() {
        let layout = PdfLayoutDocument::from_page_text_and_images(
            vec!["first".to_string(), "".to_string()],
            vec![
                vec![PageImage {
                    id: 3,
                    generation: 0,
                    extractable: true,
                }],
                Vec::new(),
            ],
        );

        assert_eq!(layout.pages.len(), 2);
        assert!(layout.has_text());
        assert!(layout.has_images());
        assert_eq!(layout.pages[0].number, 1);
        assert_eq!(layout.pages[0].text(), "first");
        assert_eq!(layout.pages[0].blocks[0].kind, PdfBlockKind::Paragraph);
        assert_eq!(layout.pages[0].images[0].id, 3);
        assert!(!layout.pages[1].has_text());
    }
}
