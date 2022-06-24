use axum::routing::get;
use axum::Router;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

trait Renderable {
    fn render(&self, prefix: String) -> String;
}

#[derive(Serialize, Deserialize, Debug)]
struct Form {
    title: String,
    description: String,
    groups: Vec<QuestionSet>,
}

impl Renderable for Form {
    fn render(&self, prefix: String) -> String {
        format!(
            "<html style=\"font-family=sans-serif\"><body><h1>{title}</h1><p>{desc}</p><div class=\"content\"><pre>{qs}</pre></div></body></html>",
            title=self.title,
            desc=self.description,
            qs=self.groups
                .iter()
                .enumerate()
                .map(|(idx, q)| q.render(format!("{prefix}-{idx}")))
                .join("\n")
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct QuestionSet {
    title: String,
    description: String,
    questions: Vec<Question>,
}

impl Renderable for QuestionSet {
    fn render(&self, prefix: String) -> String {
        format!(
            "<h2>{title}</h2><p>{desc}</p><form>{qs}</form>",
            title = self.title,
            desc = self.description,
            qs = self
                .questions
                .iter()
                .enumerate()
                .map(|(idx, q)| q.render(format!("{prefix}-{idx}")))
                .join("\n</br>\n")
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Question {
    title: String,
    spec: QuestionType,
}

impl Renderable for Question {
    fn render(&self, prefix: String) -> String {
        format!("<h3>{}</h3>{}", self.title, self.spec.render(prefix))
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum QuestionType {
    DiscreteNumeric {
        bounds: (i8, i8),
        num_descriptions: HashMap<i8, String>,
    },
    ContinousNumeric {
        bounds: Option<(f32, f32)>,
    },
    SingleChoice {
        answers: Vec<String>,
        custom_answer: bool,
    },
    MultipleChoice {
        answers: Vec<String>,
        custom_answer: bool,
    },
    Text {
        is_long: bool,
    },
}

impl Renderable for QuestionType {
    fn render(&self, id: String) -> String {
        match self {
            QuestionType::Text { is_long } => {
                if *is_long {
                    format!(r#"<textarea id="{id}"></textarea>"#)
                } else {
                    format!(r#"<input type="text" id="{id}">"#)
                }
            }
            QuestionType::ContinousNumeric {
                bounds: Some((min, max)),
            } => format!(r#"<input type="range" min="{min}" max="{max}" class="slider" id="{id}">"#),
            QuestionType::ContinousNumeric { bounds: None } => format!(r#"<input type="number" id="{id}">"#),
            QuestionType::DiscreteNumeric {
                bounds: (min, max),
                num_descriptions,
            } => (*min..=*max)
                .map(|val| {
                    let stringified = num_descriptions
                        .get(&val)
                        .map(|v| format!("{val} ({v})"))
                        .unwrap_or_else(|| val.to_string());
                    format!(r#"<input type="radio" name="{id}" id="{id}-{val}" value="{val}"><label for="{id}-{val}">{stringified}</label>"#)
                })
                .join("\n"),
            QuestionType::SingleChoice {answers, custom_answer } => {
                let custom_string = format!(r#"
<input type="radio" name="{id}" id="{id}-c" value=""><input type="text" id="{id}-t" onkeyup="document.getElementById('{id}-c').setAttribute('value', this.value)">"#);
                answers
                .iter()
                .enumerate()
                .map(|(idx, val)| {
                    format!(r#"<input type="radio" name="{id}" id="{id}-{idx}" value="{val}"><label for="{id}-{idx}">{val}</label>"#)
                })
                .join("\n") + if *custom_answer {&custom_string} else {""}},
            QuestionType::MultipleChoice {answers, custom_answer } => {
                let custom_string = format!(r#"
<input type="checkbox" name="{id}" id="{id}-c" value=""><input type="text" id="{id}-t" onkeyup="document.getElementById('{id}-c').setAttribute('value', this.value)">"#);
                answers
                .iter()
                .enumerate()
                .map(|(idx, val)| {
                    format!(r#"<input type="checkbox" name="{id}" id="{id}-{idx}" value="{val}"><label for="{id}-{idx}">{val}</label>"#)
                })
                .join("\n") + if *custom_answer {&custom_string} else {""}},
        }
    }
}

#[tokio::main]
async fn main() {
    run().await
}

async fn run() {
    let questions = vec![
        Question {
            title: "Why would you do this?".to_string(),
            spec: QuestionType::Text { is_long: true },
        },
        Question {
            title: "How much is the fish?".to_string(),
            spec: QuestionType::ContinousNumeric { bounds: None },
        },
        Question {
            title: "What do you want?".to_string(),
            spec: QuestionType::DiscreteNumeric {
                bounds: (1, 10),
                num_descriptions: HashMap::from([
                    (1, "NOPE!".to_string()),
                    (10, "YESSSSH!!!!".to_string()),
                ]),
            },
        },
        Question {
            title: "What do you want?".to_string(),
            spec: QuestionType::SingleChoice {
                answers: vec!["Pizza", "Ravioli", "MAOAM"]
                    .iter()
                    .map(|v| v.to_string())
                    .collect(),
                custom_answer: true,
            },
        },
        Question {
            title: "What do you want?".to_string(),
            spec: QuestionType::MultipleChoice {
                answers: vec!["Pizza", "Ravioli", "MAOAM"]
                    .iter()
                    .map(|v| v.to_string())
                    .collect(),
                custom_answer: true,
            },
        },
    ];

    let groups = vec![QuestionSet {
        title: "Set 1".to_string(),
        description: "".to_string(),
        questions,
    }];

    let html = format!(
        "{}",
        Form {
            title: "This is Survey speaking!".to_string(),
            description: "Hello, I am survey.".to_string(),
            groups
        }
        .render("i".to_string())
    );

    //let data = serde_yaml::from_str::<Form>(
        //&std::fs::read_to_string("test.yml").expect("Could not find testfile"),
    //)
    //.expect("Parsing failed!");

    //println!("{data:#?}");

    //let html = data.render("i".to_string());

    let app = Router::new().route("/", get(|| async { axum::response::Html::from(html) }));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
