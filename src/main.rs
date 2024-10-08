use std::fs::File;
use std::io::Write;
use std::path::Path;

use promptuity::prompts::Input;
use promptuity::themes::FancyTheme;
use promptuity::{Promptuity, Term};

use speedymd::config::read_from_json;
use speedymd::frontmatter::{self, FrontmatterValue};

use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
struct Args;

fn main() -> Result<(), promptuity::Error> {
    let _ = Args::parse();

    let mut term = Term::default();
    let mut theme = FancyTheme::default();
    let mut p = Promptuity::new(&mut term, &mut theme);
    let config = read_from_json().unwrap();
    let ext = config.ext;
    let output_path = config.output_path;
    let frontmatter_fields = config.frontmatter_fields;

    p.term().clear()?;

    p.with_intro("Setup your markdown file speedily.").begin()?;

    let filename = p.prompt(Input::new("Enter the filename").with_placeholder("filename"))?;

    let mut frontmatter_values = Vec::<FrontmatterValue>::with_capacity(frontmatter_fields.len());
    if frontmatter_fields.len() > 0 {
        p.step("Fill in the following frontmatter fields.")?;

        // Iterate over the frontmatter fields and prompt the user for input
        for field in &frontmatter_fields {
            if field.field_type == "object" {
                field
                    .properties
                    .iter()
                    .try_for_each::<_, Result<(), promptuity::Error>>(|prop_field| {
                        let value = frontmatter::extract_value_with_prompt(
                            &mut p,
                            prop_field,
                            Some(&field.name),
                        )?;
                        frontmatter_values.push(value);
                        Ok(())
                    })?;
            } else {
                let value = frontmatter::extract_value_with_prompt(&mut p, field, None)?;
                frontmatter_values.push(value);
            }
        }
    }

    let path: std::path::PathBuf =
        Path::new(&(format!("{}/{}", output_path, filename))).with_extension(&ext);
    let display = path.display();

    let mut file = match File::create(&path) {
        Ok(file) => file,
        Err(why) => panic!("Couldn't create {}: {}.", display, why),
    };

    if frontmatter_values.len() > 0 {
        let frontmatter = frontmatter::generate_format_yaml(&frontmatter_values);
        match file.write_all(frontmatter.as_bytes()) {
            Ok(()) => (),
            Err(why) => panic!("Couldn't write to {}: {}.", display, why),
        }
    }

    p.with_outro(format!(
        "Successfully generated {}.{}🎉 Happy writing!",
        filename, ext
    ))
    .finish()?;

    Ok(())
}
