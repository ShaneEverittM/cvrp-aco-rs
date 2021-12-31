use std::{
    fs::File,
    io::Read,
    num::ParseFloatError,
    str::FromStr,
};

use anyhow::{anyhow, Result};
use strum::EnumString;
use yoos::collections::Matrix;

pub type NomResult<I, O> = nom::IResult<I, O, nom::error::VerboseError<I>>;

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Problem {
    pub name: String,
    pub comment: String,
    pub problem_type: ProblemType,
    pub dimension: usize,
    pub edge_weight_type: EdgeWeightType,
    pub capacity: usize,
    pub adjacency_matrix: Matrix,
    pub demands: Vec<usize>,
}

impl Problem {
    fn parse(i: &str) -> NomResult<&str, Self> {
        // Local use statement so as not to clutter top of file, we need many
        use nom::{
            error::ParseError,
            IResult,
            combinator::{map_res, map_parser},
            bytes::complete::{tag, take_until1},
            sequence::{terminated, delimited, preceded, tuple, separated_pair},
            character::complete::{digit1, space0, space1, line_ending, not_line_ending},
            multi::count,
        };

        /******************************/
        /*        Helper parsers      */
        /******************************/

        /// Applies the inner parser, then consumes any number of spaces then a line ending.
        fn trailing_ws<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
            where
                F: FnMut(&'a str) -> IResult<&'a str, O, E> + 'a,
                E: ParseError<&'a str> + 'a
        {
            terminated(inner, preceded(space0, line_ending))
        }

        /// Parses the key, a colon, the uses the provided parser to parse the value, then parses
        /// any number of spaces then a line ending.
        fn key_then<'a, F, O, E>(
            key: &'a str,
            value_parser: F,
        ) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
            where
                F: FnMut(&'a str) -> IResult<&'a str, O, E> + 'a,
                E: ParseError<&'a str> + 'a,
                O: 'a
        {
            trailing_ws(preceded(tuple((tag(key), tag(" : "))), value_parser))
        }

        /******************************/
        /*       Actual Parsing       */
        /******************************/

        // Problem name
        let (i, name) = key_then("NAME", not_line_ending)(i)?;

        // Comment about problem
        let (i, comment) = key_then(
            "COMMENT",
            delimited(tag("("), take_until1(")"), tag(")")),
        )(i)?;

        // Type, mapped to ProblemType
        let (i, problem_type) = map_parser(
            key_then("TYPE", not_line_ending),
            ProblemType::parse,
        )(i)?;

        // Dimension, mapped to usize
        let (i, dimension) = map_res(
            key_then("DIMENSION", not_line_ending),
            usize::from_str,
        )(i)?;

        // Edge weight type, mapped to EdgeWeightType
        let (i, edge_weight_type) = map_parser(
            key_then("EDGE_WEIGHT_TYPE", not_line_ending),
            EdgeWeightType::parse,
        )(i)?;

        // Capacity, mapped to usize
        let (i, capacity) = map_res(
            key_then("CAPACITY", not_line_ending),
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
#[derive(EnumString)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum ProblemType {
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
#[derive(EnumString)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum EdgeWeightType {
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

    use anyhow::Result;

    use super::*;

    #[test]
    fn test_from_vrp() -> Result<()> {
        let problem = Problem::try_from_vrp(File::open("./inputs/A-n32-k5.vrp")?)?;

        assert_eq!(problem.name, "A-n32-k5");
        assert_eq!(problem.comment, "Augerat et al, No of trucks: 5, Optimal value: 784");
        assert_eq!(problem.problem_type, ProblemType::Cvrp);
        assert_eq!(problem.dimension, 32);
        assert_eq!(problem.edge_weight_type, EdgeWeightType::Euc2d);
        assert_eq!(problem.capacity, 100);
        assert_eq!(problem.demands[1], 19);

        Ok(())
    }
}
