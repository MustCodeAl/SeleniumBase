use thirtyfour::By;

use crate::error::SeleniumBaseError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Selector<'a> {
    LinkText(&'a str),
    PartialLinkText(&'a str),
    Css(&'a str),
    XPath(&'a str),
    Id(&'a str),
}

impl<'a> Selector<'a> {
    pub fn to_by(self) -> Result<By, SeleniumBaseError> {
        match self {
            Self::Css(value) if !value.trim().is_empty() => Ok(By::Css(value.to_owned())),
            Self::XPath(value) if !value.trim().is_empty() => Ok(By::XPath(value.to_owned())),
            Self::Id(value) if !value.trim().is_empty() => Ok(By::Id(value.to_owned())),
            Self::LinkText(value) if !value.trim().is_empty() => {
                Ok(By::LinkText(value.to_owned()))
            }
            Self::PartialLinkText(value) if !value.trim().is_empty() => {
                Ok(By::PartialLinkText(value.to_owned()))
            }
            _ => Err(SeleniumBaseError::InvalidSelector(
                "selector value cannot be empty".to_owned(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn variant_name(by: By) -> String {
        format!("{:?}", by)
    }

    #[test]
    fn css_selector_to_by() {
        let by = Selector::Css("#id").to_by().unwrap();
        assert!(variant_name(by).contains("Css"));
    }

    #[test]
    fn xpath_selector_to_by() {
        let by = Selector::XPath("//div").to_by().unwrap();
        assert!(variant_name(by).contains("XPath"));
    }

    #[test]
    fn id_selector_to_by() {
        let by = Selector::Id("user").to_by().unwrap();
        assert!(variant_name(by).contains("Id"));
    }

    #[test]
    fn link_text_selector_to_by() {
        let by = Selector::LinkText("Home").to_by().unwrap();
        assert!(variant_name(by).contains("LinkText"));
    }

    #[test]
    fn partial_link_text_selector_to_by() {
        let by = Selector::PartialLinkText("Hom").to_by().unwrap();
        assert!(variant_name(by).contains("PartialLinkText"));
    }

    #[test]
    fn empty_selector_fails() {
        assert!(Selector::Css("  ").to_by().is_err());
    }
}
