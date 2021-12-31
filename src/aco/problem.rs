use std::{
    fs::File,
    io::Read,
    str::FromStr,
    num::ParseFloatError,
};

use anyhow::{anyhow, Result};
use strum::EnumString;
use yoos::collections::Matrix;


#[allow(dead_code)]
#[derive(Debug)]
pub struct Problem {
    name: String,
    comment: String,
    problem_type: ProblemType,
    edge_weight_type: EdgeWeightType,
    dimension: usize,
    pub adjacency_matrix: Matrix,
    pub demands: Vec<usize>,
    pub capacity: usize,

}

pub type NomResult<I, O> = nom::IResult<I, O, nom::error::VerboseError<I>>;

impl Problem {
    fn parse(i: &str) -> NomResult<&str, Self> {
        // Local use statement so as not to clutter top of file, we need many
        use nom::{
            error::ParseError,
            IResult,
            combinator::{map_res, map_parser},
            bytes::complete::{tag, take_until1, take_while1},
            sequence::{terminated, delimited, preceded, tuple, separated_pair},
            character::complete::{digit1, space0, space1, line_ending},
            multi::count,
        };

        /******************************/
        /*        Helper parsers      */
        /******************************/

        // Applies the inner parser, then consumes any number of spaces then a line ending
        fn trailing_ws<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
            where
                F: FnMut(&'a str) -> IResult<&'a str, O, E> + 'a,
                E: ParseError<&'a str> + 'a
        {
            terminated(inner, preceded(space0, line_ending))
        }

        // Single word value after "<key>:"
        fn word_after<'a, E>(key: &'a str) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str, E>
            where
                E: ParseError<&'a str> + 'a
        {
            preceded(
                terminated(tag(key), tag(" : ")),
                trailing_ws(take_while1(
                    |c: char| {
                        c.is_ascii_alphanumeric() || c.is_ascii_punctuation()
                    }
                )),
            )
        }

        // Paren-delimited sentence after "<key>:"
        fn comment_after<'a, E>(key: &'a str) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str, E>
            where
                E: ParseError<&'a str> + 'a
        {
            preceded(
                terminated(tag(key), tag(" : ")),
                trailing_ws(delimited(tag("("), take_until1(")"), tag(")"))),
            )
        }

        /******************************/
        /*       Actual Parsing       */
        /******************************/

        // Problem name
        let (i, name) = word_after("NAME")(i)?;

        // Comment about problem
        let (i, comment) = comment_after("COMMENT")(i)?;

        // Type, mapped to ProblemType
        let (i, problem_type) = map_parser(
            word_after("TYPE"),
            ProblemType::parse,
        )(i)?;

        // Dimension, mapped to usize
        let (i, dimension) = map_res(
            word_after("DIMENSION"),
            usize::from_str,
        )(i)?;

        // Edge weight type, mapped to EdgeWeightType
        let (i, edge_weight_type) = map_parser(
            word_after("EDGE_WEIGHT_TYPE"),
            EdgeWeightType::parse,
        )(i)?;

        // Capacity, mapped to usize
        let (i, capacity) = map_res(
            word_after("CAPACITY"),
            usize::from_str,
        )(i)?;

        // One coordinate triplet
        let coordinate = map_res(
            trailing_ws(
                preceded(
                    tuple((space1, digit1, space1)),
                    separated_pair(digit1, space1, digit1),
                )
            ),
            |(x, y): (&str, &str)| -> Result<_, ParseFloatError> {
                Ok((x.parse::<f64>()?, y.parse::<f64>()?)) // Discard unneeded values
            },
        );

        // After the header, get exactly <dimension> tuples of 3 digits separated by spaces,
        // map them to NodeCoordinates, then Matrix
        let (i, coordinates) = preceded(
            trailing_ws(tag("NODE_COORD_SECTION")),
            count(coordinate, dimension),
        )(i)?;

        // One demand
        let demand = map_res(
            trailing_ws(preceded(tuple((digit1, space1)), digit1)),
            usize::from_str,
        );

        // After the header, get exactly <dimension> pairs of values, 
        // mapping the second of which to a demand value
        let (i, demands) = preceded(
            trailing_ws(tag("DEMAND_SECTION")),
            count(demand, dimension),
        )(i)?;

        Ok((i, Self {
            adjacency_matrix: Matrix::adjacency(coordinates),
            demands,
            capacity,
            dimension,
            problem_type,
            edge_weight_type,
            name: name.to_string(),
            comment: comment.to_string(),
        }))
    }

    pub fn try_from_vrp(mut vrp: File) -> Result<Self> {
        use nom::{Err::{Failure, Error}, Offset};
        use nom::combinator::complete;

        let mut contents = String::new();
        vrp.read_to_string(&mut contents)?;

        let result = match complete(Problem::parse)(&contents) {
            // Normal parse, return the problem
            Ok((_, problem)) => Ok(problem),

            // Error handling must happen here, since the error type has a string slice
            // into contents, so if we returned that error directly, we would have a slice
            // into the dropped contents. We can only get Failures or Errors.
            Err(Failure(err) | Error(err)) => {
                let mut message = String::from("Parsing failed: ");
                for (error_slice, err) in err.errors {
                    let offset = contents.offset(error_slice);
                    message += &format!("{:?} at position {}: '{}'", err, offset, error_slice);
                }
                Err(anyhow!(message))
            }

            // Since wrapped in complete, Incomplete is transformed into Error
            _ => unreachable!()
        };

        result
    }
}

#[non_exhaustive]
#[derive(Debug, PartialEq, EnumString)]
enum ProblemType {
    #[strum(ascii_case_insensitive)]
    Cvrp,
}

impl ProblemType {
    pub fn parse(i: &str) -> NomResult<&str, Self> {
        use nom::{
            combinator::map_res,
            bytes::complete::tag,
        };
        map_res(tag("CVRP"), ProblemType::from_str)(i)
    }
}

#[non_exhaustive]
#[derive(Debug, PartialEq, EnumString)]
enum EdgeWeightType {
    #[strum(serialize = "EUC_2D")]
    Euc2d,
}

impl EdgeWeightType {
    pub fn parse(i: &str) -> NomResult<&str, Self> {
        use nom::{
            combinator::map_res,
            bytes::complete::tag,
        };
        map_res(tag("EUC_2D"), EdgeWeightType::from_str)(i)
    }
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use crate::aco::problem::{EdgeWeightType, ProblemType};

    use crate::Problem;

    #[test]
    fn test() {
        let input = File::open("./inputs/A-n32-k5.vrp").unwrap();
        let problem = Problem::try_from_vrp(input);

        if problem.is_err() {
            eprintln!("{}", problem.as_ref().unwrap_err());
        }

        assert!(problem.is_ok());
        let problem = problem.unwrap();

        assert_eq!(problem.name, "A-n32-k5");
        assert_eq!(problem.comment, "Augerat et al, No of trucks: 5, Optimal value: 784");
        assert_eq!(problem.problem_type, ProblemType::Cvrp);
        assert_eq!(problem.dimension, 32);
        assert_eq!(problem.edge_weight_type, EdgeWeightType::Euc2d);
        assert_eq!(problem.capacity, 100);

        dbg!(problem.adjacency_matrix);

        assert_eq!(problem.demands[1], 19);
    }
}
