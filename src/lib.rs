use std::error::Error;
use std::collections::HashMap;
use std::cmp::Ordering;
use std::fs;

pub struct Config
{
    filepath: String,
    search: String,
    rank: bool,
    unique: bool,
    exact: bool,
    mappings: HashMap<String, i32>
}

impl Config
{
    pub fn build(args: impl Iterator<Item = String>) -> Result<Self, String>
    {
        let mut filepath: Option<String> = None;
        let mut search = String::new();

        let mut rank = false;
        let mut unique = false;
        let mut exact = false;

        let mut mappings: HashMap<String, i32> = HashMap::new();

        let mut args = args.peekable();
        while let Some(arg) = args.next()
        {
            if args.peek().is_none()
            {
                filepath = Some(arg);
                continue;
            }

            match &arg[..]
            {
                "-s" => search = args.next().ok_or("no search text")?,

                "-m" =>
                {
                    let mapping = args.next().ok_or("no mapping")?;
                    mappings = Self::parse_mappings(&mapping)?;
                },

                "-r" | "--rank" => rank = true,
                "-u" | "--unique" => unique = true,
                "-e" | "--exact" => exact = true,
                _ => ()
            }
        }

        let filepath = filepath.ok_or("no filepath specified")?;

        if !rank
        {
            if search.is_empty()
            {
                return Err(String::from("no search string specified"));
            }
        }

        Ok(Config{filepath, search, rank, unique, exact, mappings})
    }

    fn parse_mappings(mapping: &str) -> Result<HashMap<String, i32>, String>
    {
        let splitter = mapping.chars().next().ok_or("no splitter")?;

        let mut pairs = mapping.split(splitter).skip(1);

        let mut mappings = HashMap::new();
        while let Some(key) = pairs.next()
        {
            mappings.insert(key.to_string(),
                pairs.next().ok_or(format!("{key} has no matching value"))?
                    .parse().map_err(|error| format!("{error}"))?);
        }

        Ok(mappings)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn mappings()
    {
        let mappings = Config::parse_mappings("test6tshouldt9twork!t3t!t4");
        let mappings = mappings.expect("no mappings returned");

        assert_eq!(mappings.get("es"), Some(&6));
        assert_eq!(mappings.get("should"), Some(&9));
        assert_eq!(mappings.get("work!"), Some(&3));
        assert_eq!(mappings.get("!"), Some(&4));
    }
}

mod csv;
use csv::csv_reader::Answers;

pub fn run(config: &Config) -> Result<(), Box<dyn Error>>
{
    let file = fs::read_to_string(&config.filepath)?;

    let answers = Answers::parse(&file)?;

    if !config.rank
    {
        let replies =
        {
            if config.exact
            {
                answers.question_exact(&config.search)
            } else
            {
                answers.question(&config.search)
            }
        }.ok_or(format!("cant find {}", &config.search))?;

        if config.unique
        {
            print_unique(config, &answers, replies)
        } else
        {
            print_normal(config, replies)
        }
    } else
    {
        print_ranked(config, answers)
    }
}

fn print_normal(config: &Config, replies: Vec<&str>) -> Result<(), Box<dyn Error>>
{
    println!("{}", replies[0]);

    let no_label_replies = replies.into_iter().skip(1);
    let mode = mode(no_label_replies.clone()).ok_or("no replies given")?;
    println!("most popular: {mode}");

    if !config.mappings.is_empty()
    {
        let mapped: Vec<i32> = map_replies(no_label_replies.clone(), &config.mappings);

        let median = median(&mapped);
        let average = average(&mapped);
        println!("average: {average:.2}, median: {median:.2}");
    }

    let display_replies = no_label_replies.clone().filter(|text| !text.is_empty());
    if config.mappings.is_empty()
    {
        println!("all replies: {}", format_replies(display_replies));
    } else
    {
        let sorted_replies = sort_replies(
            display_replies.collect::<Vec<&str>>(),
            &config.mappings);

        println!("sorted replies: {}", format_replies(sorted_replies.into_iter()));
    }

    Ok(())
}

fn print_unique(
    config: &Config,
    answers: &Answers,
    replies: Vec<&str>) -> Result<(), Box<dyn Error>>
{
    for (index, uid) in replies.iter().skip(1).enumerate()
    {
        let ureplies = answers.reply(index).ok_or("uid amount doesnt match to replies")?;

        if ureplies.is_empty()
        {
            continue;
        }

        let ureplies = ureplies.iter().flatten().map(|owned| &owned[..]);

        println!("{}:", uid.trim());
        println!("{{");

        let mode = mode(ureplies.clone())
            .expect("all users should have replies");

        println!("    most popular: {mode}");

        if !config.mappings.is_empty()
        {
            let mapped = map_replies(ureplies, &config.mappings);

            let median = median(&mapped);
            let average = average(&mapped);

            println!("    average: {average:.2}, median: {median:.2}");
        }

        println!("}}\n");
    }

    Ok(())
}

fn print_ranked(config: &Config, answers: Answers) -> Result<(), Box<dyn Error>>
{
    let labels = answers.labels();
    let replies = answers.replies();

    let mut sums = vec![0; labels.len()];
    for (index, _) in labels.iter().enumerate()
    {
        for reply in replies
        {
            let mapped = reply[index].iter()
                .fold(0, |acc, current| acc+config.mappings.get(current).unwrap_or(&0));
            sums[index] += mapped;
        }
    }

    let scale = replies.len() as f64;
    let mut label_sums: Vec<(&str, f64)> = sums.iter().enumerate()
        .map(|(index, value)|
        {
            (&labels[index][..], f64::from(value.clone())/scale)
        })
        .skip(1)
        .collect();

    label_sums.sort_by(|other, current|
    {
        current.1.partial_cmp(&other.1).unwrap_or(Ordering::Less)
    });

    for sum in label_sums
    {
        println!("{}: average {:.2}", sum.0, sum.1);
    }

    Ok(())
}

fn format_replies<'a>(replies: impl Iterator<Item=&'a str>) -> String
{
    let mut out = String::new();
    for reply in replies
    {
        out.push_str(reply);
        out.push_str(", ");
    }
    out.pop();
    out.pop();

    out
}

fn mode<'a>(replies: impl Iterator<Item=&'a str>) -> Option<&'a str>
{
    let mut occurrences: HashMap<&str, u32> = HashMap::new();
    for reply in replies.filter(|text| !text.is_empty())
    {
        let current = occurrences.entry(reply).or_insert(0);
        *current += 1;
    }

    if occurrences.is_empty()
    {
        return None;
    }

    let most = occurrences.iter().fold(
        occurrences.iter().next().expect("map should not be empty"),
        |highest, current|
        {
            if current.1>highest.1
            {
                return current;
            } else
            {
                return highest;
            }
        });

    Some(most.0)
}

fn map_replies<'a>(
    replies: impl Iterator<Item=&'a str>,
    mapping: &HashMap<String, i32>) -> Vec<i32>
{
    replies.filter(|choice| mapping.get(choice.clone()).is_some())
        .map(|choice|
        {
            mapping.get(choice)
            .expect("all invalid values should be filtered").clone()
        }).collect()
}

fn sort_replies<'a>(mut replies: Vec<&'a str>, mapping: &HashMap<String, i32>) -> Vec<&'a str>
{
    replies.sort_by(|other, current|
    {
        let other = mapping.get(other.clone());
        let current = mapping.get(current.clone());
        other.cmp(&current)
    });

    replies
}

fn median(slice: &[i32]) -> f64
{
    if slice.is_empty()
    {
        return 0.0;
    }

    let amount = slice.len();

    let mut sorted: Vec<i32> = Vec::from(slice.clone());
    sorted.sort();

    let middle = amount/2;
    if amount%2==0
    {
        let upper = sorted[middle];
        let lower = sorted[middle-1];

        f64::from(upper+lower)/2.0
    } else
    {
        f64::from(sorted[middle])
    }
}

fn average(slice: &[i32]) -> f64
{
    if slice.is_empty()
    {
        return 0.0;
    }

    let amount: u32 = slice.len().try_into().expect("cant convert usize to u32");

    let total = slice.iter().fold(0, |acc, current| acc+current);

    f64::from(total)/f64::from(amount)
}