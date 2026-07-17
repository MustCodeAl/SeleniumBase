use crate::error::SeleniumBaseError;
use std::fs;
use std::path::Path;

/// Supported chart types.
#[derive(Clone, Debug)]
pub enum ChartType {
    Pie,
    Bar,
    Line,
    Area,
    Column,
}

/// A simple HTML/Chart.js chart generator.
#[derive(Clone, Debug)]
pub struct Chart {
    pub title: String,
    pub chart_type: ChartType,
    pub data: Vec<(String, i32)>,
}

impl Chart {
    pub fn new(title: &str, chart_type: ChartType) -> Self {
        Self {
            title: title.to_owned(),
            chart_type,
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
        let values: Vec<String> = self
            .data
            .iter()
            .map(|(_, value)| value.to_string())
            .collect();
        let colors: Vec<String> = self
            .data
            .iter()
            .enumerate()
            .map(|(i, _)| format!("\"{}\"", default_color(i)))
            .collect();

        let labels_json = format!("[{}]", labels.join(", "));
        let values_json = format!("[{}]", values.join(", "));
        let colors_json = format!("[{}]", colors.join(", "));

        let (chart_type, fill, index_axis) = match self.chart_type {
            ChartType::Pie => ("'pie'", "false", ""),
            ChartType::Bar => ("'bar'", "false", "indexAxis: 'x',"),
            ChartType::Line => ("'line'", "true", ""),
            ChartType::Area => ("'line'", "true", ""),
            ChartType::Column => ("'bar'", "false", "indexAxis: 'y',"),
        };

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
    type: {chart_type},
    data: {{
        labels: {labels},
        datasets: [{{
            label: '{title}',
            data: {values},
            backgroundColor: {colors},
            fill: {fill}
        }}]
    }},
    options: {{
        {index_axis}
        responsive: true
    }}
}});
</script>
</body>
</html>"#,
            title = self.title,
            chart_type = chart_type,
            labels = labels_json,
            values = values_json,
            colors = colors_json,
            fill = fill,
            index_axis = index_axis
        );

        fs::write(path.as_ref(), html)?;
        Ok(())
    }
}

/// Backwards-compatible pie chart constructor.
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
        let mut chart = Chart::new(&self.title, ChartType::Pie);
        for (label, value) in &self.data {
            chart.add_data_point(label, *value);
        }
        chart.save(path)
    }
}

fn default_color(index: usize) -> String {
    let palette = [
        "#3366cc", "#dc3912", "#ff9900", "#109618", "#990099", "#0099c6", "#dd4477", "#66aa00",
        "#b82e2e", "#316395",
    ];
    palette[index % palette.len()].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn save_pie_chart_contains_data() {
        let mut chart = Chart::new("Votes", ChartType::Pie);
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

    #[test]
    fn save_bar_chart_contains_data() {
        let mut chart = Chart::new("Sales", ChartType::Bar);
        chart.add_data_point("Q1", 100);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("chart.html");
        chart.save(&path).unwrap();

        let html = fs::read_to_string(&path).unwrap();
        assert!(html.contains("'bar'"));
        assert!(html.contains("100"));
    }
}
