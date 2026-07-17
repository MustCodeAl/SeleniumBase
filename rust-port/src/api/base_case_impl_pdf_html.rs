// Additional BaseCase methods for PDF, HTML parsing, and themed tours.

impl BaseCase {
    /// Captures the current page as a PDF using CDP `Page.printToPDF`.
    pub async fn print_to_pdf(&self, filename: &str) -> Result<(), SeleniumBaseError> {
        let response = self
            .session
            .execute_cdp_with_params(
                "Page.printToPDF",
                serde_json::json!({
                    "printBackground": true,
                    "preferCSSPageSize": true,
                }),
            )
            .await?;

        let data = response
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SeleniumBaseError::Unsupported("PDF data missing from CDP response".to_owned()))?;

        let bytes = BASE64_STANDARD
            .decode(data)
            .map_err(|e| SeleniumBaseError::Unsupported(format!("Failed to decode PDF: {e}")))?;

        pdf::save_pdf_bytes(&bytes, filename)?;
        Ok(())
    }

    /// Alias for `print_to_pdf`.
    pub async fn save_as_pdf(&self, filename: &str) -> Result<(), SeleniumBaseError> {
        self.print_to_pdf(filename).await
    }

    /// Extracts text from a PDF file on disk.
    pub fn get_pdf_text(&self, filename: &str) -> Result<String, SeleniumBaseError> {
        pdf::extract_text_from_file(filename)
    }

    /// Asserts that `text` appears inside the given PDF.
    pub fn assert_pdf_text(&self, filename: &str, text: &str) -> Result<(), SeleniumBaseError> {
        let content = self.get_pdf_text(filename)?;
        if !content.contains(text) {
            return Err(SeleniumBaseError::AssertionFailed(format!(
                "PDF '{}' did not contain text '{}'",
                filename, text
            )));
        }
        Ok(())
    }

    /// Returns a BeautifulSoup-style parser for the current page source.
    pub async fn get_beautiful_soup_object(&self) -> Result<BeautifulSoup, SeleniumBaseError> {
        let source = self.get_page_source().await?;
        Ok(BeautifulSoup::parse(&source))
    }

    /// Creates a tour and assigns a visual theme.
    pub async fn create_tour_with_theme(
        &mut self,
        name: &str,
        theme: TourTheme,
    ) -> Result<(), SeleniumBaseError> {
        self.tour = Some(crate::api::tour::Tour::new(name).with_theme(theme));
        Ok(())
    }
}
