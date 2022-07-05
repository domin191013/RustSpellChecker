use hyper::service::{make_service_fn, service_fn};
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashSet;
use std::cmp::min;
use core::convert::Infallible;
use hyper::{Body, Request, Response, StatusCode};
// use hyper::header::{Headers, AccessControlAllowOrigin};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct CheckResponse {
  pub suggestions: Vec<String>,
  pub correct: bool,
}

pub fn assemble_word_hash_set<'a>(contents: &'a str) -> HashSet<&'a str> {
    let mut word_set = HashSet::new();

    for (i, line) in contents.lines().enumerate() {
		if i >= 45 {
		
			let split_line = line.split(" ");
			let vec = split_line.collect::<Vec<&str>>();

			for item in vec {
				let item = item.trim_matches('\\');            
				word_set.insert(item);
			};
		}
    }

    word_set
}

fn get_dictionary_hash_set<'a>() -> String {
    let mut dictionary = String::new();
    let mut dictionary_file = File::open("dictionary.txt")
        .expect("File Not Found :(");

    let _ = &dictionary_file.read_to_string(&mut dictionary)
        .expect("Something went wrong :( Could not read the file");
    
    dictionary
}

pub fn get_distance_of_words<'a, 'b>(s1: &'a String, s2: &'b String) -> u32 {
    let rows = s2.chars().count() + 1;
    let columns = s1.chars().count() + 1;

    let mut matrix = Vec::new();

    for _ in 0..rows {
        matrix.push(Vec::new());
    }
    
    for mut row in &mut matrix {
        for _ in 0..columns {
            row.push(0);
        }
    }

    for num in 0..columns {
        matrix[0][num] = num;
    }

    for num in 0..rows {
        matrix[num][0] = num;
    }

    for i in 1..rows {
        for j in 1..columns {
            if s2[i-1..i] == s1[j-1..j] {
                matrix[i][j] = matrix[i-1][j-1];
            }
            else {
                matrix[i][j] = 1 + min(matrix[i-1][j-1], min(matrix[i-1][j], matrix[i][j-1]));
            }
        }
    }

    matrix[rows-1][columns-1] as u32
}

fn get_suggestions(word: &str) -> CheckResponse {
    let dictionary = get_dictionary_hash_set();
    let dictionary = assemble_word_hash_set(&dictionary);
    let mut replacements = Vec::new();
	let mut replacements_distance_is_two = Vec::new();

    if dictionary.contains(word.to_lowercase().as_str()) || dictionary.contains(word) {
        let res = CheckResponse{
            suggestions: Vec::new(),
            correct: true
        };
        return res;
    }

    for word_dict in dictionary {
		let word_and_rank: Vec<&str> = word_dict.split(" ").collect();
		let edit_distance = get_distance_of_words(&word_and_rank[0].to_string(), &word.to_string());
						
		if edit_distance <= 1 {
			replacements.push(word_and_rank[0].to_owned());
		}
		else if edit_distance <= 2 {
			replacements_distance_is_two.push(word_and_rank[0].to_owned());

            if replacements.len() + replacements_distance_is_two.len() > 10 {
                break;
            }
		}
	}

    replacements.append(&mut replacements_distance_is_two);

    let mut res = CheckResponse{
        suggestions: replacements,
        correct: false
    };
    if res.suggestions.len() > 10 {
        res.suggestions = res.suggestions.as_slice()[..10].to_vec();
    }

    res
}

async fn web_service(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    println!("Web Service.....");
    let path = req.uri().path();
    let path = path.split("/").collect::<Vec<&str>>();
    let response = Response::builder()
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Headers", "*")
        .header("Access-Control-Allow-Methods", "POST, GET, OPTIONS");
    
    if path.len() < 3 || path[1] != "spellcheck" {
        return Ok(
            response
            .status(StatusCode::OK)
            .body("Spell Checker gate way : please use /spellcheck/{My Word} gateway".into())
            .unwrap()
        )
    }

    let word = path[2];
    let res:CheckResponse = get_suggestions(word);
    let json = serde_json::to_string(&res).unwrap();

    println!("{}", json);
    if res.correct == false && res.suggestions.len() == 0 {
        return Ok(
            response
            .status(StatusCode::NOT_FOUND)
            .body(json.into())
            .unwrap()
        ) 
    }
    Ok(
        response
        .status(StatusCode::OK)
        .body(json.into())
        .unwrap()
    ) 
}

#[tokio::main]
async fn main() {
    println!("Started running spell checker project...");
    
    let addr = "127.0.0.1:31337".parse().unwrap();

    let make_service = make_service_fn(|_| async { 
        Ok::<_, Infallible>(service_fn(web_service)) 
    });

    // let mut headers = Headers::new();
    // headers.set(
    //     AccessControlAllowOrigin::Any
    // );
    let server = hyper::Server::bind(&addr).serve(make_service);

    println!("Listening on http://localhost:31337");

    let _ = server.await;
}
