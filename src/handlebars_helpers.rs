handlebars_helper!(format_number: |x: f64| format!("{:.*}", 2, x));
handlebars_helper!(has_mods: |mods: array| mods.len() > 0);
