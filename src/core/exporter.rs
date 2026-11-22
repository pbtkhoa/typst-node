use typst::layout::{Abs, PagedDocument};

use crate::error::CoreError;

pub fn export_pdf(doc: &PagedDocument) -> Result<Vec<u8>, CoreError> {
  let options = typst_pdf::PdfOptions::default();
  typst_pdf::pdf(doc, &options).map_err(|e| CoreError::ExportFailed(format!("{e:?}")))
}

pub fn export_svg(doc: &PagedDocument) -> Result<Vec<String>, CoreError> {
  let mut pages = Vec::with_capacity(doc.pages.len());
  for page in &doc.pages {
    pages.push(typst_svg::svg(page));
  }
  Ok(pages)
}

pub fn export_svg_merged(doc: &PagedDocument) -> Result<String, CoreError> {
  let gap = Abs::pt(20.0);
  Ok(typst_svg::svg_merged(doc, gap))
}

pub fn export_png(
  doc: &PagedDocument,
  ppi: f64,
  page_numbers: Option<&[u32]>,
) -> Result<Vec<Vec<u8>>, CoreError> {
  // 72 pt = 1 inch -> pixel_per_pt_ratio = ppi / 72.0
  let pixel_per_pt_ratio = (ppi / 72.0) as f32;

  let target_indices: Vec<usize> = if let Some(pages) = page_numbers {
    pages
      .iter()
      .filter_map(|&p| if p > 0 { Some((p - 1) as usize) } else { None })
      .filter(|&idx| idx < doc.pages.len())
      .collect()
  } else {
    (0..doc.pages.len()).collect()
  };

  let mut buffers = Vec::with_capacity(target_indices.len());
  for idx in target_indices {
    let page = &doc.pages[idx];
    let pixmap = typst_render::render(page, pixel_per_pt_ratio);
    let png_bytes = pixmap
      .encode_png()
      .map_err(|e| CoreError::ExportFailed(format!("PNG encode error: {e:?}")))?;
    buffers.push(png_bytes);
  }

  Ok(buffers)
}
