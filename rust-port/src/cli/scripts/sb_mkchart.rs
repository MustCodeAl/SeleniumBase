use std::fs::File;
use std::io::Write;

pub fn make_chart(filename: &str) {
    let mut file = File::create(filename).unwrap();
    let content = r#"<!DOCTYPE html>
<html>
<head>
    <title>Test Results Chart</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
</head>
<body>
    <canvas id="results"></canvas>
    <script>
        new Chart(document.getElementById('results'), {
            type: 'bar',
            data: {
                labels: ['Passed', 'Failed', 'Skipped'],
                datasets: [{
                    label: 'Test Results',
                    data: [10, 1, 2],
                    backgroundColor: ['green', 'red', 'orange']
                }]
            }
        });
    </script>
</body>
</html>"#;
    file.write_all(content.as_bytes()).unwrap();
    println!("Created chart at {}", filename);
}
