use std::{
    fs::File,
    io::Read,
    str::FromStr,
    num::ParseIntError,
};

use anyhow::{anyhow, Result};
use strum::EnumString;

pub use matrix::Matrix;

mod matrix;


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
            combinator::{opt, map, map_res, map_parser},
            bytes::complete::{tag, take_until1, take_while1},
            sequence::{delimited, preceded, tuple, pair},
            character::complete::{digit1, space1, line_ending},
            multi::{many0, separated_list1, count},
        };

        /******************************/
        /*        Helper parsers      */
        /******************************/

        // End of lines sometimes have trailing spaces. 
        // This is a macro since taking function as argument
        // is overly complicated when we can just copy paste.
        macro_rules! trailing_ws {
            ($inner:expr) => {
                nom::sequence::terminated(
                    $inner, 
                    |input| preceded(opt(many0(tag(" "))), line_ending)(input)
                )
            } 
        }

        // Key followed by colon, macro for same reason as above.
        macro_rules! key_then {
            ($key:expr, $inner:expr) => {
                nom::sequence::preceded(
                    pair(tag($key), tag(" : ")),
                    $inner, 
                )
            } 
        }

        // Single word value after "<key>:"
        let word_after = |key| {
            let word = take_while1(
                |c: char| {
                    c.is_ascii_alphanumeric() || c.is_ascii_punctuation()
                }
            );
            key_then!(key, trailing_ws!(word))
        };

        // Paren-delimited sentence after "<key>:"
        let comment_after = |key| {
            let between_parens = delimited(tag("("), take_until1(")"), tag(")"));
            key_then!(key, trailing_ws!(between_parens))
        };

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

        // After the header, get exactly <dimension> tuples of 3 digits separated by spaces,
        // map them to NodeCoordinates, then Matrix
        let (i, adjacency_matrix) = map(
            preceded(
                trailing_ws!(tag("NODE_COORD_SECTION")),
                count(
                    map_res(
                        map(
                            trailing_ws!(
                            preceded(space1, tuple((digit1, space1, digit1, space1, digit1)))
                        ),
                            |f| (f.2, f.4), // Discard unneeded values
                        ),
                        NodeCoordinate::from_tuple,
                    ),
                    dimension,
                ),
            ),
            Matrix::from,
        )(i)?;

        // After the header, get exactly <dimension> pairs of values, 
        // mapping the second of which to a demand value
        let (i, demands) = preceded(
            trailing_ws!(tag("DEMAND_SECTION")),
            count(
                map_res(
                    trailing_ws!(separated_list1(space1, digit1)),
                    |v: Vec<&str>| -> Result<_, ParseIntError> {
                        usize::from_str(v[1])
                    },
                ),
                dimension,
            ),
        )(i)?;

        Ok((i, Self {
            adjacency_matrix,
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

#[derive(Default)]
struct NodeCoordinate {
    x: f64,
    y: f64,
}

impl NodeCoordinate {
    pub fn from_tuple(v: (&str, &str)) -> Result<Self> {
        Ok(NodeCoordinate { x: v.0.parse()?, y: v.1.parse()? })
    }

    pub fn distance_from(&self, other: &NodeCoordinate) -> f64 {
        ((other.y - self.y).powi(2) + (other.x - self.x).powi(2)).sqrt()
    }
}

impl From<Vec<NodeCoordinate>> for Matrix {
    fn from(coordinates: Vec<NodeCoordinate>) -> Self {
        let mut matrix = Matrix::new(coordinates.len());
        for (i, a) in coordinates.iter().enumerate() {
            for (j, b) in coordinates.iter().enumerate() {
                matrix[i][j] = a.distance_from(b);
            }
        }
        matrix
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
