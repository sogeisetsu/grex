/*
 * Copyright © 2019-today Peter M. Stahl pemistahl@gmail.com
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either expressed or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#![allow(deprecated)]

use crate::regexp::feature::Feature;
use crate::regexp::{RegExp, RegExpConfig};
use itertools::Itertools;
use std::io::ErrorKind;
use std::path::PathBuf;

/// This struct builds regular expressions from user-provided test cases.
pub struct RegExpBuilder {
    test_cases: Vec<String>,
    config: RegExpConfig,
}

impl RegExpBuilder {
    /// Specifies the test cases to build the regular expression from.
    ///
    /// The test cases need not be sorted because `RegExpBuilder` sorts them internally.
    ///
    /// ⚠ Panics if `test_cases` is empty.
    pub fn from<T: Clone + Into<String>>(test_cases: &[T]) -> Self {
        if test_cases.is_empty() {
            panic!("No test cases have been provided for regular expression generation");
        }
        Self {
            test_cases: test_cases.iter().cloned().map(|it| it.into()).collect_vec(),
            config: RegExpConfig::new(),
        }
    }

    /// Specifies a text file containing test cases to build the regular expression from.
    ///
    /// The test cases need not be sorted because `RegExpBuilder` sorts them internally.
    ///
    /// Each test case needs to be on a separate line.
    /// Lines may be ended with either a newline (`\n`) or
    /// a carriage return with a line feed (`\r\n`).
    /// The final line ending is optional.
    ///
    /// ⚠ Panics if:
    /// - the file cannot be found
    /// - the file's encoding is not valid UTF-8 data
    /// - the file cannot be opened because of conflicting permissions
    pub fn from_file<T: Into<PathBuf>>(file_path: T) -> Self {
        match std::fs::read_to_string(file_path.into()) {
            Ok(file_content) => Self {
                test_cases: file_content.lines().map(|it| it.to_string()).collect_vec(),
                config: RegExpConfig::new(),
            },
            Err(error) => match error.kind() {
                ErrorKind::NotFound => panic!("The specified file could not be found"),
                ErrorKind::InvalidData => {
                    panic!("The specified file's encoding is not valid UTF-8")
                }
                ErrorKind::PermissionDenied => {
                    panic!("Permission denied: The specified file could not be opened")
                }
                _ => panic!("{}", error),
            },
        }
    }

    /// Tells `RegExpBuilder` to convert any Unicode decimal digit to character class `\d`.
    ///
    /// This method takes precedence over
    /// [`with_conversion_of_words`](Self::with_conversion_of_words) if both are set.
    /// Decimal digits are converted to `\d`, the remaining word characters to `\w`.
    ///
    /// This method takes precedence over
    /// [`with_conversion_of_non_whitespace`](Self::with_conversion_of_non_whitespace) if both are set.
    /// Decimal digits are converted to `\d`, the remaining non-whitespace characters to `\S`.
    pub fn with_conversion_of_digits(&mut self) -> &mut Self {
        self.config.is_digit_converted = true;
        self
    }

    /// Tells `RegExpBuilder` to convert any character which is not
    /// a Unicode decimal digit to character class `\D`.
    ///
    /// This method takes precedence over
    /// [`with_conversion_of_non_words`](Self::with_conversion_of_non_words) if both are set.
    /// Non-digits which are also non-word characters are converted to `\D`.
    ///
    /// This method takes precedence over
    /// [`with_conversion_of_non_whitespace`](Self::with_conversion_of_non_whitespace) if both are set.
    /// Non-digits which are also non-space characters are converted to `\D`.
    pub fn with_conversion_of_non_digits(&mut self) -> &mut Self {
        self.config.is_non_digit_converted = true;
        self
    }

    /// Tells `RegExpBuilder` to convert any Unicode whitespace character to character class `\s`.
    ///
    /// This method takes precedence over
    /// [`with_conversion_of_non_digits`](Self::with_conversion_of_non_digits) if both are set.
    /// Whitespace characters are converted to `\s`, the remaining non-digit characters to `\D`.
    ///
    /// This method takes precedence over
    /// [`with_conversion_of_non_words`](Self::with_conversion_of_non_words) if both are set.
    /// Whitespace characters are converted to `\s`, the remaining non-word characters to `\W`.
    pub fn with_conversion_of_whitespace(&mut self) -> &mut Self {
        self.config.is_space_converted = true;
        self
    }

    /// Tells `RegExpBuilder` to convert any character which is not
    /// a Unicode whitespace character to character class `\S`.
    pub fn with_conversion_of_non_whitespace(&mut self) -> &mut Self {
        self.config.is_non_space_converted = true;
        self
    }

    /// Tells `RegExpBuilder` to convert any Unicode word character to character class `\w`.
    ///
    /// This method takes precedence over
    /// [`with_conversion_of_non_digits`](Self::with_conversion_of_non_digits) if both are set.
    /// Word characters are converted to `\w`, the remaining non-digit characters to `\D`.
    ///
    /// This method takes precedence over
    /// [`with_conversion_of_non_whitespace`](Self::with_conversion_of_non_whitespace) if both are set.
    /// Word characters are converted to `\w`, the remaining non-space characters to `\S`.
    pub fn with_conversion_of_words(&mut self) -> &mut Self {
        self.config.is_word_converted = true;
        self
    }

    /// Tells `RegExpBuilder` to convert any character which is not
    /// a Unicode word character to character class `\W`.
    ///
    /// This method takes precedence over
    /// [`with_conversion_of_non_whitespace`](Self::with_conversion_of_non_whitespace) if both are set.
    /// Non-words which are also non-space characters are converted to `\W`.
    pub fn with_conversion_of_non_words(&mut self) -> &mut Self {
        self.config.is_non_word_converted = true;
        self
    }

    /// Tells `RegExpBuilder` to detect repeated non-overlapping substrings and
    /// to convert them to `{min,max}` quantifier notation.
    pub fn with_conversion_of_repetitions(&mut self) -> &mut Self {
        self.config.is_repetition_converted = true;
        self
    }

    /// Tells `RegExpBuilder` to enable case-insensitive matching of test cases
    /// so that letters match both upper and lower case.
    pub fn with_case_insensitive_matching(&mut self) -> &mut Self {
        self.config.is_case_insensitive_matching = true;
        self
    }

    /// Tells `RegExpBuilder` to replace non-capturing groups by capturing ones.
    pub fn with_capturing_groups(&mut self) -> &mut Self {
        self.config.is_capturing_group_enabled = true;
        self
    }

    /// Tells `RegExpBuilder` which conversions should be performed during
    /// regular expression generation. The available conversion features
    /// are listed in the [`Feature`](./enum.Feature.html#variants) enum.
    ///
    /// ⚠ Panics if `features` is empty.
    #[deprecated(since = "1.3.0", note = "This method will be removed in 1.4.0.")]
    pub fn with_conversion_of(&mut self, features: &[Feature]) -> &mut Self {
        if features.is_empty() {
            panic!("No conversion features have been provided for regular expression generation");
        }
        self.config.conversion_features = features.to_vec();
        self
    }

    /// Specifies the minimum quantity of substring repetitions to be converted if
    /// [`with_conversion_of_repetitions`](Self::with_conversion_of_repetitions) is set.
    ///
    /// If the quantity is not explicitly set with this method, a default value of 1 will be used.
    ///
    /// ⚠ Panics if `quantity` is zero.
    pub fn with_minimum_repetitions(&mut self, quantity: u32) -> &mut Self {
        if quantity == 0 {
            panic!("Quantity of minimum repetitions must not be zero");
        }
        self.config.minimum_repetitions = quantity;
        self
    }

    /// Specifies the minimum length a repeated substring must have in order to be converted if
    /// [`with_conversion_of_repetitions`](Self::with_conversion_of_repetitions) is set.
    ///
    /// If the length is not explicitly set with this method, a default value of 1 will be used.
    ///
    /// ⚠ Panics if `length` is zero.
    pub fn with_minimum_substring_length(&mut self, length: u32) -> &mut Self {
        if length == 0 {
            panic!("Minimum substring length must not be zero");
        }
        self.config.minimum_substring_length = length;
        self
    }

    /// Tells `RegExpBuilder` to convert non-ASCII characters to unicode escape sequences.
    /// The parameter `use_surrogate_pairs` specifies whether to convert astral code planes
    /// (range `U+010000` to `U+10FFFF`) to surrogate pairs.
    pub fn with_escaping_of_non_ascii_chars(&mut self, use_surrogate_pairs: bool) -> &mut Self {
        self.config.is_non_ascii_char_escaped = true;
        self.config.is_astral_code_point_converted_to_surrogate = use_surrogate_pairs;
        self
    }

    /// Tells `RegExpBuilder` to produce a nicer looking regular expression in verbose mode.
    pub fn with_verbose_mode(&mut self) -> &mut Self {
        self.config.is_verbose_mode_enabled = true;
        self
    }

    /// Tells `RegExpBuilder` to remove the caret anchor '^' from the resulting regular
    /// expression, thereby allowing to match the test cases also when they do not occur
    /// at the start of a string.
    pub fn without_start_anchor(&mut self) -> &mut Self {
        self.config.is_start_anchor_disabled = true;
        self
    }

    /// Tells `RegExpBuilder` to remove the dollar sign anchor '$' from the resulting regular
    /// expression, thereby allowing to match the test cases also when they do not occur
    /// at the end of a string.
    pub fn without_end_anchor(&mut self) -> &mut Self {
        self.config.is_end_anchor_disabled = true;
        self
    }

    /// Tells `RegExpBuilder` to remove the caret and dollar sign anchors from the resulting
    /// regular expression, thereby allowing to match the test cases also when they occur
    /// within a larger string that contains other content as well.
    pub fn without_anchors(&mut self) -> &mut Self {
        self.config.is_start_anchor_disabled = true;
        self.config.is_end_anchor_disabled = true;
        self
    }

    /// Tells `RegExpBuilder` to provide syntax highlighting for the resulting regular expression.
    ///
    /// ⚠ This method may only be used if the resulting regular expression is meant to
    /// be printed to the console. The regex string representation returned from enabling
    /// this setting cannot be fed into the [*regex*](https://crates.io/crates/regex) crate.
    pub fn with_syntax_highlighting(&mut self) -> &mut Self {
        self.config.is_output_colorized = true;
        self
    }

    /// Builds the actual regular expression using the previously given settings.
    /// Every generated regular expression is surrounded by the anchors `^` and `$`
    /// so that substrings not being part of the test cases are not matched accidentally.
    pub fn build(&mut self) -> String {
        RegExp::from(&mut self.test_cases, &self.config).to_string()
    }
}
