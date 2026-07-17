use crate::error::SeleniumBaseError;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Default)]
pub struct PieChart {
    pub title: String,
    pub data: Vec<(String, i32)>,
}

impl PieChart {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_owned(),
            data: Vec::new(),
        }
    }

    pub fn add_data_point(&mut self, label: &str, value: i32) {
        self.data.push((label.to_owned(), value));
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), SeleniumBaseError> {
        let labels: Vec<String> = self
            .data
            .iter()
            .map(|(label, _)| format!("\"{}\"", label.replace('"', "\\\"")))
            .collect();
        let values: Vec<String> = self.data.iter().map(|(_, value)| value.to_string()).collect();
        let colors: Vec<String> = self
            .data
            .iter()
            .enumerate()
            .map(|(i, _)| format!("\"{}\"", default_color(i)))
            .collect();

        let labels_json = format!("[{}]", labels.join(", "));
        let values_json = format!("[{}]", values.join(", "));
        let colors_json = format!("[{}]", colors.join(", "));

        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>{title}</title>
<script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
</head>
<body>
<h1>{title}</h1>
<canvas id="chart"></canvas>
<script>
new Chart(document.getElementById('chart'), {{
    type: 'pie',
    data: {{
        labels: {labels},
        datasets: [{{
            data: {values},
            backgroundColor: {colors}
        }}]
    }}
}});
</script>
</body>
</html>"#,
            title = self.title,
            labels = labels_json,
            values = values_json,
            colors = colors_json
        );

        fs::write(path.as_ref(), html).map_err(|e| {
            SeleniumBaseError::InvalidConfig(format!(
                "failed to write chart '{}': {e}",
                path.as_ref().display()
            ))
        })?;
        Ok(())
    }
}

fn default_color(index: usize) -> String {
    let palette = [
        "#3366cc", "#dc3912", "#ff9900", "#109618", "#990099", "#0099c6",
        "#dd4477", "#66aa00", "#b82e2e", "#316395",
    ];
    palette[index % palette.len()].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn save_chart_contains_data() {
        let mut chart = PieChart::new("Votes");
        chart.add_data_point("A", 10);
        chart.add_data_point("B", 20);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("chart.html");
        chart.save(&path).unwrap();

        let html = fs::read_to_string(&path).unwrap();
        assert!(html.contains("Votes"));
        assert!(html.contains("\"A\""));
        assert!(html.contains("10"));
        assert!(html.contains("chart.js"));
        assert!(html.contains("'pie'"));
    }
}
