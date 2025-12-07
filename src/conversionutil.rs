use unit_conversions::{length, mass, temperature, time, volume};

#[derive(Debug)]
pub enum ConversionError {
    UnknownUnit(String),
    IncompatibleUnits(String, String),
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::UnknownUnit(unit) => write!(f, "Unknown unit: {}", unit),
            ConversionError::IncompatibleUnits(from, to) => {
                write!(f, "Incompatible units: {} -> {}", from, to)
            }
        }
    }
}

impl std::error::Error for ConversionError {}

pub type Result<T> = std::result::Result<T, ConversionError>;

pub fn convert(value: f64, current_unit: &str, desired_unit: &str) -> Result<f64> {
    // Try length conversions
    if let Ok(result) = try_length_conversion(value, current_unit, desired_unit) {
        return Ok(result);
    }
    
    // Try mass conversions
    if let Ok(result) = try_mass_conversion(value, current_unit, desired_unit) {
        return Ok(result);
    }
    
    // Try temperature conversions
    if let Ok(result) = try_temperature_conversion(value, current_unit, desired_unit) {
        return Ok(result);
    }
    
    // Try time conversions
    if let Ok(result) = try_time_conversion(value, current_unit, desired_unit) {
        return Ok(result);
    }
    
    // Try volume conversions
    if let Ok(result) = try_volume_conversion(value, current_unit, desired_unit) {
        return Ok(result);
    }
    
    Err(ConversionError::IncompatibleUnits(
        current_unit.to_string(),
        desired_unit.to_string(),
    ))
}

fn try_length_conversion(value: f64, from: &str, to: &str) -> Result<f64> {
    dbg!("length_conversion");
    dbg!(value);
    dbg!(from);
    dbg!(to);
    let meters = match from.to_lowercase().as_str() {
        "m" | "meter" | "meters" => value,
        "km" | "kilometer" | "kilometers" => length::kilometres::to_metres(value),
        "cm" | "centimeter" | "centimeters" => length::centimetres::to_metres(value),
        "mm" | "millimeter" | "millimeters" => length::millimetres::to_metres(value),
        "mi" | "mile" | "miles" => length::miles::to_metres(value),
        "yd" | "yard" | "yards" => length::yards::to_metres(value),
        "ft" | "foot" | "feet" => length::feet::to_metres(value),
        "in" | "inch" | "inches" => length::inches::to_metres(value),
        _ => return Err(ConversionError::UnknownUnit(from.to_string())),
    };
    
    let result = match to.to_lowercase().as_str() {
        "m" | "meter" | "meters" => meters,
        "km" | "kilometer" | "kilometers" => length::metres::to_kilometres(meters),
        "cm" | "centimeter" | "centimeters" => length::metres::to_centimetres(meters),
        "mm" | "millimeter" | "millimeters" => length::metres::to_millimetres(meters),
        "mi" | "mile" | "miles" => length::metres::to_miles(meters),
        "yd" | "yard" | "yards" => length::metres::to_yards(meters),
        "ft" | "foot" | "feet" => length::metres::to_feet(meters),
        "in" | "inch" | "inches" => length::metres::to_inches(meters),
        _ => return Err(ConversionError::UnknownUnit(to.to_string())),
    };
    
    Ok(result)
}

fn try_mass_conversion(value: f64, from: &str, to: &str) -> Result<f64> {
    let kilograms = match from.to_lowercase().as_str() {
        "kg" | "kilogram" | "kilograms" => value,
        "g" | "gram" | "grams" => mass::grams::to_kilograms(value),
        "mg" | "milligram" | "milligrams" => mass::milligrams::to_kilograms(value),
        "lb" | "pound" | "pounds" => mass::pounds::to_kilograms(value),
        "oz" | "ounce" | "ounces" => mass::ounces::to_kilograms(value),
        _ => return Err(ConversionError::UnknownUnit(from.to_string())),
    };
    
    let result = match to.to_lowercase().as_str() {
        "kg" | "kilogram" | "kilograms" => kilograms,
        "g" | "gram" | "grams" => mass::kilograms::to_grams(kilograms),
        "mg" | "milligram" | "milligrams" => mass::kilograms::to_milligrams(kilograms),
        "lb" | "pound" | "pounds" => mass::kilograms::to_pounds(kilograms),
        "oz" | "ounce" | "ounces" => mass::kilograms::to_ounces(kilograms),
        _ => return Err(ConversionError::UnknownUnit(to.to_string())),
    };
    
    Ok(result)
}

fn try_temperature_conversion(value: f64, from: &str, to: &str) -> Result<f64> {
    let celsius = match from.to_lowercase().as_str() {
        "c" | "celsius" => value,
        "f" | "fahrenheit" => temperature::fahrenheit::to_celsius(value),
        "k" | "kelvin" => temperature::kelvin::to_celsius(value),
        _ => return Err(ConversionError::UnknownUnit(from.to_string())),
    };
    
    let result = match to.to_lowercase().as_str() {
        "c" | "celsius" => celsius,
        "f" | "fahrenheit" => temperature::celsius::to_fahrenheit(celsius),
        "k" | "kelvin" => temperature::celsius::to_kelvin(celsius),
        _ => return Err(ConversionError::UnknownUnit(to.to_string())),
    };
    
    Ok(result)
}

fn try_time_conversion(value: f64, from: &str, to: &str) -> Result<f64> {
    let seconds = match from.to_lowercase().as_str() {
        "s" | "sec" | "second" | "seconds" => value,
        "min" | "minute" | "minutes" => time::minutes::to_seconds(value),
        "h" | "hr" | "hour" | "hours" => time::hours::to_seconds(value),
        "d" | "day" | "days" => time::days::to_seconds(value),
        _ => return Err(ConversionError::UnknownUnit(from.to_string())),
    };
    
    let result = match to.to_lowercase().as_str() {
        "s" | "sec" | "second" | "seconds" => seconds,
        "min" | "minute" | "minutes" => time::seconds::to_minutes(seconds),
        "h" | "hr" | "hour" | "hours" => time::seconds::to_hours(seconds),
        "d" | "day" | "days" => time::seconds::to_days(seconds),
        _ => return Err(ConversionError::UnknownUnit(to.to_string())),
    };
    
    Ok(result)
}

fn try_volume_conversion(value: f64, from: &str, to: &str) -> Result<f64> {
    let liters = match from.to_lowercase().as_str() {
        "l" | "liter" | "liters" => value,
        "ml" | "milliliter" | "milliliters" => volume::millilitres::to_litres(value),
        "gal" | "gallon" | "gallons" => volume::gallons::to_litres(value),
        _ => return Err(ConversionError::UnknownUnit(from.to_string())),
    };
    
    let result = match to.to_lowercase().as_str() {
        "l" | "liter" | "liters" => liters,
        "ml" | "milliliter" | "milliliters" => volume::litres::to_millilitres(liters),
        "gal" | "gallon" | "gallons" => volume::litres::to_gallons(liters),
        _ => return Err(ConversionError::UnknownUnit(to.to_string())),
    };
    
    Ok(result)
}
