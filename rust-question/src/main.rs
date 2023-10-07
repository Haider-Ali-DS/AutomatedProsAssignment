use core::panic;

fn get_operators(character: String) -> Option<String> {
    match character.as_str() {
        "a" => Some(String::from("+")),
        "b" => Some(String::from("-")),
        "c" => Some(String::from("*")),
        "d" => Some(String::from("/")),
        "e" => Some(String::from("(")),
        "f" => Some(String::from(")")),
        _ => None,
    }
}

fn apply_arithmetic(operator: String, x: u16, y: u16) -> Option<u16>{
    match operator.as_str() {
        "+" => Some(x + y),
        "-" => Some(x - y),
        "*" => Some(x * y),
        "/" => Some(x / y),
        _ => None
    }
}

fn translate_input(input: &str) -> Vec<String> {
    let mut translated_data: Vec<String> = vec![];
    let mut number_builder = String::from("");
    for (index, char) in input.chars().enumerate() {
        if char.is_numeric() {
            number_builder = format!("{}{}", number_builder, char);
        } else {
            if !number_builder.is_empty() {
                translated_data.push(number_builder.clone());
                number_builder = "".into();
            }
            let Some(operator) = get_operators(char.to_string()) else{
                panic!("Invalid character found {}", char);
            };
            translated_data.push(operator.into())
        }
        if index == (input.len() - 1) && !number_builder.is_empty(){
            translated_data.push(number_builder.clone());
        }
    }
    translated_data
}

fn solve_inner(mut data: Vec<String>) -> Vec<String> {
    let later_opening_bracket = data.iter().rposition(|d| d == "(").unwrap_or_else(|| panic!("Invalid equation"));
    let first_closing_bracket = data.iter().position(|d| d == ")").unwrap_or_else(|| panic!("Invalid equation"));
    let inner_equation: Vec<String> = data[later_opening_bracket+1..first_closing_bracket].into();
    let result = solve_outer(inner_equation);
    data.splice(later_opening_bracket..first_closing_bracket+1, result);
    return data;
}

fn solve_outer(mut data: Vec<String>) -> Vec<String> {
    while data.len() > 1 {
        let x: u16 = data.get(0).unwrap_or_else(|| panic!("Invalid equation")).parse().unwrap();
        let operator = data.get(1).unwrap_or_else(|| panic!("Invalid equation"));
        let y: u16 = data.get(2).unwrap_or_else(|| panic!("Invalid equation")).parse().unwrap();
        let result = apply_arithmetic(operator.into(), x, y).unwrap_or_else(|| panic!("Invalid equation"));
        data.splice(0..=2, vec![result.to_string()]);
    }
    data
    
}

fn solve_equation(input: &str) -> i32 {
    let mut translated_data = translate_input(input);
    while translated_data.contains(&String::from("(")) || translated_data.contains(&String::from(")")) {
        let solved_equation = solve_inner(translated_data);
        translated_data = solved_equation;
    }
    while translated_data.len() > 1 {
        let solved_equation = solve_outer(translated_data);
        translated_data = solved_equation;
    }
    translated_data[0].parse().unwrap()
}

fn main() {
    let input_1 = "3a2c4";
    let result_1 = 20;
    assert_eq!(solve_equation(input_1), result_1);
    let input_2 = "32a2d2";
    let result_2 = 17;
    assert_eq!(solve_equation(input_2), result_2);
    let input_3 = "500a10b66c32";
    let result_3 = 14208;
    assert_eq!(solve_equation(input_3), result_3);
    let input_4 = "3ae4c66fb32";
    let result_4 = 235;
    assert_eq!(solve_equation(input_4), result_4);
    let input_5 = "3c4d2aee2a4c41fc4f";
    let result_5 = 990;
    assert_eq!(solve_equation(input_5), result_5);
    println!("All tests passed");
}
