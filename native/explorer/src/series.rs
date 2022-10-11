use crate::{
    datatypes::{ExDate, ExDateTime},
    encoding, ExDataFrame, ExSeries, ExplorerError,
};

use polars::prelude::*;
use rand::seq::IteratorRandom;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use rustler::{Encoder, Env, Term};
use std::result::Result;

pub(crate) fn to_series_collection(s: Vec<ExSeries>) -> Vec<Series> {
    s.into_iter().map(|c| c.resource.0.clone()).collect()
}

pub(crate) fn to_ex_series_collection(s: Vec<Series>) -> Vec<ExSeries> {
    s.into_iter().map(ExSeries::new).collect()
}

#[rustler::nif]
pub fn s_as_str(data: ExSeries) -> Result<String, ExplorerError> {
    Ok(format!("{:?}", data.resource.0))
}

macro_rules! init_method {
    ($name:ident, $type:ty) => {
        #[rustler::nif]
        pub fn $name(name: &str, val: Vec<Option<$type>>) -> ExSeries {
            ExSeries::new(Series::new(name, val.as_slice()))
        }
    };
}

init_method!(s_new_i64, i64);
init_method!(s_new_bool, bool);
init_method!(s_new_f64, f64);
init_method!(s_new_str, String);

#[rustler::nif]
pub fn s_new_date32(name: &str, val: Vec<Option<ExDate>>) -> ExSeries {
    ExSeries::new(
        Series::new(
            name,
            val.iter()
                .map(|d| d.map(|d| d.into()))
                .collect::<Vec<Option<i32>>>(),
        )
        .cast(&DataType::Date)
        .unwrap(),
    )
}

#[rustler::nif]
pub fn s_new_date64(name: &str, val: Vec<Option<ExDateTime>>) -> ExSeries {
    ExSeries::new(
        Series::new(
            name,
            val.iter()
                .map(|dt| dt.map(|dt| dt.into()))
                .collect::<Vec<Option<i64>>>(),
        )
        .cast(&DataType::Datetime(TimeUnit::Microseconds, None))
        .unwrap(),
    )
}

#[rustler::nif]
pub fn s_name(data: ExSeries) -> Result<String, ExplorerError> {
    Ok(data.resource.0.name().to_string())
}

#[rustler::nif]
pub fn s_rename(data: ExSeries, name: &str) -> Result<ExSeries, ExplorerError> {
    let mut s = data.resource.0.clone();
    s.rename(name);
    Ok(ExSeries::new(s))
}

#[rustler::nif]
pub fn s_dtype(data: ExSeries) -> Result<String, ExplorerError> {
    let s = &data.resource.0;
    let dt = s.dtype().to_string();
    Ok(dt)
}

#[rustler::nif]
pub fn s_slice(data: ExSeries, offset: i64, length: usize) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let series = s.slice(offset, length);
    Ok(ExSeries::new(series))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_append(data: ExSeries, other: ExSeries) -> Result<ExSeries, ExplorerError> {
    let mut s = data.resource.0.clone();
    let s1 = &other.resource.0;
    s.append(s1)?;
    Ok(ExSeries::new(s))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_filter(data: ExSeries, filter: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &filter.resource.0;
    if let Ok(ca) = s1.bool() {
        let series = s.filter(ca)?;
        Ok(ExSeries::new(series))
    } else {
        Err(ExplorerError::Other("Expected a boolean mask".into()))
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_add(data: ExSeries, other: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &other.resource.0;
    Ok(ExSeries::new(s + s1))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_add_number(data: ExSeries, number: i64) -> Result<ExSeries, ExplorerError> {
    let s: &Series = &data.resource.0;
    let s1: Series = Series::new("", &[number]);
    Ok(ExSeries::new(s + &s1))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_sub(data: ExSeries, other: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &other.resource.0;
    Ok(ExSeries::new(s - s1))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_mul(data: ExSeries, other: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &other.resource.0;
    Ok(ExSeries::new(s * s1))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_div(data: ExSeries, other: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &other.resource.0;
    Ok(ExSeries::new(s / s1))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_quotient(data: ExSeries, other: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &other.resource.0;
    let div = s.checked_div(s1)?;

    Ok(ExSeries::new(div))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_remainder(data: ExSeries, other: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &other.resource.0;
    let div = s.checked_div(s1)?;
    let mult = s1 * &div;
    let result = s - &mult;

    Ok(ExSeries::new(result))
}

#[rustler::nif]
pub fn s_head(data: ExSeries, length: Option<usize>) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.head(length)))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_tail(data: ExSeries, length: Option<usize>) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.tail(length)))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_sort(data: ExSeries, reverse: bool) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.sort(reverse)))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_argsort(data: ExSeries, reverse: bool) -> Result<Vec<Option<u32>>, ExplorerError> {
    let s = &data.resource.0;
    Ok(s.argsort(SortOptions {
        descending: reverse,
        nulls_last: false,
    })
    .into_iter()
    .collect::<Vec<Option<u32>>>())
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_distinct(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let unique = s.take(&s.arg_unique()?)?;
    Ok(ExSeries::new(unique))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_unordered_distinct(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let unique = s.unique()?;
    Ok(ExSeries::new(unique))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_value_counts(data: ExSeries) -> Result<ExDataFrame, ExplorerError> {
    let s: &Series = &data.resource.0;
    let mut df = s.value_counts(true, true)?;
    let df = df
        .try_apply("counts", |s: &Series| s.cast(&DataType::Int64))?
        .clone();
    Ok(ExDataFrame::new(df))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_slice_by_indices(data: ExSeries, indices: Vec<u32>) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let idx = UInt32Chunked::from_vec("idx", indices);
    let s1 = s.take(&idx)?;
    Ok(ExSeries::new(s1))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_is_null(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.is_null().into_series()))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_is_not_null(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.is_not_null().into_series()))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_take_every(data: ExSeries, n: usize) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = s.take_every(n);
    Ok(ExSeries::new(s1))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_series_equal(
    data: ExSeries,
    other: ExSeries,
    null_equal: bool,
) -> Result<bool, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &other.resource.0;
    let result = if null_equal {
        s.series_equal_missing(s1)
    } else {
        s.series_equal(s1)
    };
    Ok(result)
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_eq(data: ExSeries, rhs: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &rhs.resource.0;
    Ok(ExSeries::new(s.equal(s1).unwrap().into_series()))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_neq(data: ExSeries, rhs: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &rhs.resource.0;
    Ok(ExSeries::new(s.not_equal(s1).unwrap().into_series()))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_gt(data: ExSeries, rhs: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &rhs.resource.0;
    Ok(ExSeries::new(s.gt(s1).unwrap().into_series()))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_gt_eq(data: ExSeries, rhs: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &rhs.resource.0;
    Ok(ExSeries::new(s.gt_eq(s1).unwrap().into_series()))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_lt(data: ExSeries, rhs: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &rhs.resource.0;
    Ok(ExSeries::new(s.lt(s1).unwrap().into_series()))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_lt_eq(data: ExSeries, rhs: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &rhs.resource.0;
    Ok(ExSeries::new(s.lt_eq(s1).unwrap().into_series()))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_and(lhs: ExSeries, rhs: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &lhs.resource.0;
    let s1 = &rhs.resource.0;
    let and = s.bool()? & s1.bool()?;
    Ok(ExSeries::new(and.into_series()))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_or(lhs: ExSeries, rhs: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &lhs.resource.0;
    let s1 = &rhs.resource.0;
    let or = s.bool()? | s1.bool()?;
    Ok(ExSeries::new(or.into_series()))
}

#[rustler::nif]
pub fn s_len(data: ExSeries) -> Result<usize, ExplorerError> {
    let s = &data.resource.0;
    Ok(s.len())
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_fill_none(data: ExSeries, strategy: &str) -> Result<ExSeries, ExplorerError> {
    let strat = match strategy {
        "backward" => FillNullStrategy::Backward(None),
        "forward" => FillNullStrategy::Forward(None),
        "min" => FillNullStrategy::Min,
        "max" => FillNullStrategy::Max,
        "mean" => FillNullStrategy::Mean,
        s => {
            return Err(ExplorerError::Other(format!(
                "Strategy {} not supported",
                s
            )))
        }
    };

    let s = &data.resource.0;
    let s1 = s.fill_null(strat)?;
    Ok(ExSeries::new(s1))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_fill_none_with_int(data: ExSeries, strategy: i64) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s = s.i64()?.fill_null_with_values(strategy)?.into_series();
    Ok(ExSeries::new(s))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_fill_none_with_float(data: ExSeries, strategy: f64) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s = s.f64()?.fill_null_with_values(strategy)?.into_series();
    Ok(ExSeries::new(s))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_fill_none_with_bin(data: ExSeries, strategy: &str) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s = s.utf8()?.fill_null_with_values(strategy)?.into_series();
    Ok(ExSeries::new(s))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_rolling_sum(
    data: ExSeries,
    window_size: usize,
    weights: Option<Vec<f64>>,
    min_periods: Option<usize>,
    center: bool,
) -> Result<ExSeries, ExplorerError> {
    let s: &Series = &data.resource.0;
    let opts = rolling_opts(window_size, weights, min_periods, center);
    let s1 = s.rolling_sum(opts.into())?;
    Ok(ExSeries::new(s1))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_rolling_mean(
    data: ExSeries,
    window_size: usize,
    weights: Option<Vec<f64>>,
    min_periods: Option<usize>,
    center: bool,
) -> Result<ExSeries, ExplorerError> {
    let s: &Series = &data.resource.0;
    let opts = rolling_opts(window_size, weights, min_periods, center);
    let s1 = s.rolling_mean(opts.into())?;
    Ok(ExSeries::new(s1))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_rolling_max(
    data: ExSeries,
    window_size: usize,
    weights: Option<Vec<f64>>,
    min_periods: Option<usize>,
    center: bool,
) -> Result<ExSeries, ExplorerError> {
    let s: &Series = &data.resource.0;
    let opts = rolling_opts(window_size, weights, min_periods, center);
    let s1 = s.rolling_max(opts.into())?;
    Ok(ExSeries::new(s1))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_rolling_min(
    data: ExSeries,
    window_size: usize,
    weights: Option<Vec<f64>>,
    min_periods: Option<usize>,
    center: bool,
) -> Result<ExSeries, ExplorerError> {
    let s: &Series = &data.resource.0;
    let opts = rolling_opts(window_size, weights, min_periods, center);
    let s1 = s.rolling_min(opts.into())?;
    Ok(ExSeries::new(s1))
}

// Used for rolling functions - also see "expressions" module
pub fn rolling_opts(
    window_size: usize,
    weights: Option<Vec<f64>>,
    min_periods: Option<usize>,
    center: bool,
) -> RollingOptions {
    let min_periods = if let Some(mp) = min_periods {
        mp
    } else {
        window_size
    };
    let window_size_duration = Duration::new(window_size as i64);

    RollingOptions {
        window_size: window_size_duration,
        weights,
        min_periods,
        center,
        ..Default::default()
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_to_list(env: Env, data: ExSeries) -> Result<Term, ExplorerError> {
    Ok(encoding::list_from_series(data, env))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_sum(env: Env, data: ExSeries) -> Result<Term, ExplorerError> {
    let s = &data.resource.0;
    match s.dtype() {
        DataType::Boolean => Ok(s.sum::<i64>().encode(env)),
        DataType::Int8
        | DataType::UInt8
        | DataType::Int16
        | DataType::UInt16
        | DataType::Int32
        | DataType::UInt32
        | DataType::Int64 => Ok(s.sum::<i64>().encode(env)),
        DataType::Float32 | DataType::Float64 => Ok(s.sum::<f64>().encode(env)),
        dt => panic!("sum/1 not implemented for {:?}", dt),
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_min(env: Env, data: ExSeries) -> Result<Term, ExplorerError> {
    let s: &Series = &data.resource.0;
    match s.dtype() {
        DataType::Int8
        | DataType::UInt8
        | DataType::Int16
        | DataType::UInt16
        | DataType::Int32
        | DataType::UInt32
        | DataType::Int64 => Ok(s.min::<i64>().encode(env)),
        DataType::Float32 | DataType::Float64 => Ok(s.min::<f64>().encode(env)),
        DataType::Date => Ok(s.min::<i32>().map(ExDate::from).encode(env)),
        DataType::Datetime(TimeUnit::Microseconds, None) => {
            Ok(s.min::<i64>().map(ExDateTime::from).encode(env))
        }
        dt => panic!("min/1 not implemented for {:?}", dt),
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_max(env: Env, data: ExSeries) -> Result<Term, ExplorerError> {
    let s: &Series = &data.resource.0;
    match s.dtype() {
        DataType::Int8
        | DataType::UInt8
        | DataType::Int16
        | DataType::UInt16
        | DataType::Int32
        | DataType::UInt32
        | DataType::Int64 => Ok(s.max::<i64>().encode(env)),
        DataType::Float32 | DataType::Float64 => Ok(s.max::<f64>().encode(env)),
        DataType::Date => Ok(s.max::<i32>().map(ExDate::from).encode(env)),
        DataType::Datetime(TimeUnit::Microseconds, None) => {
            Ok(s.max::<i64>().map(ExDateTime::from).encode(env))
        }
        dt => panic!("max/1 not implemented for {:?}", dt),
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_mean(env: Env, data: ExSeries) -> Result<Term, ExplorerError> {
    let s = &data.resource.0;
    match s.dtype() {
        DataType::Boolean => Ok(s.mean().encode(env)),
        DataType::Int8
        | DataType::UInt8
        | DataType::Int16
        | DataType::UInt16
        | DataType::Int32
        | DataType::UInt32
        | DataType::Int64
        | DataType::Float32
        | DataType::Float64 => Ok(s.mean().encode(env)),
        dt => panic!("mean/1 not implemented for {:?}", dt),
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_median(env: Env, data: ExSeries) -> Result<Term, ExplorerError> {
    let s = &data.resource.0;
    match s.dtype() {
        DataType::Int8
        | DataType::UInt8
        | DataType::Int16
        | DataType::UInt16
        | DataType::Int32
        | DataType::UInt32
        | DataType::Int64
        | DataType::Float32
        | DataType::Float64 => Ok(s.median().encode(env)),
        dt => panic!("median/1 not implemented for {:?}", dt),
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_var(env: Env, data: ExSeries) -> Result<Term, ExplorerError> {
    let s = &data.resource.0;
    match s.dtype() {
        DataType::Int8
        | DataType::UInt8
        | DataType::Int16
        | DataType::UInt16
        | DataType::Int32
        | DataType::UInt32
        | DataType::Int64 => Ok(s.i64().unwrap().var(1).encode(env)),
        DataType::Float32 | DataType::Float64 => Ok(s.f64().unwrap().var(1).encode(env)),
        dt => panic!("var/1 not implemented for {:?}", dt),
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_std(env: Env, data: ExSeries) -> Result<Term, ExplorerError> {
    let s = &data.resource.0;
    match s.dtype() {
        DataType::Int8
        | DataType::UInt8
        | DataType::Int16
        | DataType::UInt16
        | DataType::Int32
        | DataType::UInt32
        | DataType::Int64 => Ok(s.i64().unwrap().std(1).encode(env)),
        DataType::Float32 | DataType::Float64 => Ok(s.f64().unwrap().std(1).encode(env)),
        dt => panic!("std/1 not implemented for {:?}", dt),
    }
}

#[rustler::nif]
pub fn s_get(env: Env, data: ExSeries, idx: usize) -> Result<Term, ExplorerError> {
    let s = &data.resource.0;
    Ok(encoding::term_from_value(s.get(idx), env))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_cum_sum(data: ExSeries, reverse: bool) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.cumsum(reverse)))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_cum_max(data: ExSeries, reverse: bool) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.cummax(reverse)))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_cum_min(data: ExSeries, reverse: bool) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.cummin(reverse)))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_quantile<'a>(
    env: Env<'a>,
    data: ExSeries,
    quantile: f64,
    strategy: &str,
) -> Result<Term<'a>, ExplorerError> {
    let s = &data.resource.0;
    let dtype = s.dtype();
    let strategy = parse_quantile_interpol_options(strategy);
    match dtype {
        DataType::Date => match s.date()?.quantile(quantile, strategy)? {
            None => Ok(None::<ExDate>.encode(env)),
            Some(days) => Ok(ExDate::from(days as i32).encode(env)),
        },
        DataType::Datetime(TimeUnit::Microseconds, None) => {
            match s.datetime()?.quantile(quantile, strategy)? {
                None => Ok(None::<ExDateTime>.encode(env)),
                Some(microseconds) => Ok(ExDateTime::from(microseconds as i64).encode(env)),
            }
        }
        _ => Ok(encoding::term_from_value(
            s.quantile_as_series(quantile, strategy)?
                .cast(dtype)?
                .get(0),
            env,
        )),
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_peak_max(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.peak_max().into_series()))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_peak_min(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.peak_min().into_series()))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_reverse(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.reverse()))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_n_unique(data: ExSeries) -> Result<usize, ExplorerError> {
    let s = &data.resource.0;
    Ok(s.n_unique()?)
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_pow(data: ExSeries, exponent: f64) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s = s
        .cast(&DataType::Float64)?
        .f64()?
        .apply(|v| v.powf(exponent))
        .into_series();
    Ok(ExSeries::new(s))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_int_pow(data: ExSeries, exponent: u32) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s = s.i64()?.apply(|v| v.pow(exponent)).into_series();
    Ok(ExSeries::new(s))
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_cast(data: ExSeries, to_type: &str) -> Result<ExSeries, ExplorerError> {
    let s: &Series = &data.resource.0;
    let dtype = cast_str_to_dtype(to_type)?;
    Ok(ExSeries::new(s.cast(&dtype)?))
}

pub fn cast_str_to_dtype(str_type: &str) -> Result<DataType, ExplorerError> {
    match str_type {
        "float" => Ok(DataType::Float64),
        "integer" => Ok(DataType::Int64),
        "date" => Ok(DataType::Date),
        "datetime" => Ok(DataType::Datetime(TimeUnit::Microseconds, None)),
        "boolean" => Ok(DataType::Boolean),
        "string" => Ok(DataType::Utf8),
        _ => Err(ExplorerError::Other(String::from("Cannot cast to type"))),
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_seedable_random_indices(
    length: usize,
    n_samples: usize,
    replacement: bool,
    seed: u64,
) -> Vec<usize> {
    let mut rng: Pcg64 = SeedableRng::seed_from_u64(seed);
    let range: Vec<usize> = (0..length).collect();
    if replacement {
        (0..n_samples).map(|_| rng.gen_range(0..length)).collect()
    } else {
        range
            .iter()
            .choose_multiple(&mut rng, n_samples)
            .iter()
            .map(|x| **x)
            .collect()
    }
}

pub fn parse_quantile_interpol_options(strategy: &str) -> QuantileInterpolOptions {
    match strategy {
        "nearest" => QuantileInterpolOptions::Nearest,
        "lower" => QuantileInterpolOptions::Lower,
        "higher" => QuantileInterpolOptions::Higher,
        "midpoint" => QuantileInterpolOptions::Midpoint,
        "linear" => QuantileInterpolOptions::Linear,
        _ => QuantileInterpolOptions::Nearest,
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn s_coalesce(data: ExSeries, other: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s1 = &data.resource.0;
    let s2 = &other.resource.0;
    let coalesced = s1.zip_with(&s1.is_not_null(), s2)?;
    Ok(ExSeries::new(coalesced))
}
