use csv::ReaderBuilder;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    al::tools::{ThreadContext, Tool},
    AppState,
};

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TrialBalanceRow {
    pub account_number: String,
    pub account_name: String,
    pub category: String,
    pub balance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VarianceData {
    pub account: String,
    pub category: String,
    pub current_balance: f64,
    pub prior_balance: f64,
    pub variance_absolute: f64,
    pub variance_percent: f64,
    pub is_material: bool,
    pub priority: String, // "High", "Medium", "Low"
}

// ============================================================================
// Tool: ParseTrialBalance
// ============================================================================

#[derive(Clone, Debug)]
pub struct ParseTrialBalance;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParseTrialBalanceInput {
    /// The trial balance data in CSV format or as structured JSON
    pub data: String,
    /// The period end date in YYYY-MM-DD format
    pub period_end: String,
    /// Optional label: "current" or "prior"
    pub period_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseTrialBalanceOutput {
    pub success: bool,
    pub message: String,
    pub period_id: Option<i32>,
    pub accounts_parsed: usize,
    pub validation_errors: Vec<String>,
}

#[async_trait::async_trait]
impl Tool for ParseTrialBalance {
    const NAME: &'static str = "parse_trial_balance";
    const DESCRIPTION: &'static str = r#"
    Parse and validate a trial balance CSV file or JSON data.

    This tool accepts trial balance data in CSV format or structured JSON and validates it.
    The CSV should have columns: account_number, account_name, category, balance

    Example CSV format:
    ```
    account_number,account_name,category,balance
    1000,Cash and Cash Equivalents,Current Asset,12800000
    1100,Restricted Cash,Current Asset,2500000
    2100,Accounts Payable,Current Liability,450000
    ```

    Example JSON input:
    ```json
    {
        "data": "account_number,account_name,category,balance\n1000,Cash,Current Asset,12800000",
        "period_end": "2025-09-30",
        "period_label": "current"
    }
    ```

    The tool will:
    1. Validate the data format
    2. Parse the accounts
    3. Store them in the database
    4. Return a summary with any validation errors
    "#;

    type ToolInput = ParseTrialBalanceInput;
    type ToolOutput = ParseTrialBalanceOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        use chrono::NaiveDate;

        let mut validation_errors = Vec::new();

        // Parse the period end date
        let period_end = NaiveDate::parse_from_str(&input.period_end, "%Y-%m-%d")
            .map_err(|e| {
                cja::color_eyre::eyre::eyre!("Invalid period_end date format: {}", e)
            })?;

        // Parse CSV data
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(input.data.as_bytes());

        let mut accounts = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            match result {
                Ok(record) => {
                    if record.len() < 4 {
                        validation_errors.push(format!(
                            "Row {}: Expected 4 columns, found {}",
                            idx + 1,
                            record.len()
                        ));
                        continue;
                    }

                    let account_number = record.get(0).unwrap_or("").to_string();
                    let account_name = record.get(1).unwrap_or("").to_string();
                    let category = record.get(2).unwrap_or("").to_string();
                    let balance_str = record.get(3).unwrap_or("0");

                    // Validate required fields
                    if account_number.is_empty() {
                        validation_errors.push(format!("Row {}: account_number is empty", idx + 1));
                        continue;
                    }
                    if account_name.is_empty() {
                        validation_errors.push(format!("Row {}: account_name is empty", idx + 1));
                        continue;
                    }
                    if category.is_empty() {
                        validation_errors.push(format!("Row {}: category is empty", idx + 1));
                        continue;
                    }

                    // Parse balance
                    let balance: f64 = balance_str
                        .replace(',', "")
                        .replace('$', "")
                        .trim()
                        .parse()
                        .unwrap_or_else(|_| {
                            validation_errors.push(format!(
                                "Row {}: Invalid balance value '{}'",
                                idx + 1,
                                balance_str
                            ));
                            0.0
                        });

                    accounts.push(TrialBalanceRow {
                        account_number,
                        account_name,
                        category,
                        balance,
                    });
                }
                Err(e) => {
                    validation_errors.push(format!("Row {}: CSV parse error: {}", idx + 1, e));
                }
            }
        }

        if accounts.is_empty() && validation_errors.is_empty() {
            return Ok(ParseTrialBalanceOutput {
                success: false,
                message: "No accounts found in the data".to_string(),
                period_id: None,
                accounts_parsed: 0,
                validation_errors: vec!["No valid data rows found".to_string()],
            });
        }

        // Store in database
        let pool = &app_state.db;

        // Create or get financial period
        let period_id: i32 = sqlx::query_scalar(
            r#"
            INSERT INTO financial_periods (period_end, company_id, thread_id)
            VALUES ($1, 1, $2)
            ON CONFLICT (period_end, company_id)
            DO UPDATE SET thread_id = $2
            RETURNING id
            "#,
        )
        .bind(period_end)
        .bind(context.thread.thread_id)
        .fetch_one(pool)
        .await?;

        // Insert accounts
        for account in &accounts {
            sqlx::query(
                r#"
                INSERT INTO accounts (period_id, account_number, account_name, category, balance)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (period_id, account_number)
                DO UPDATE SET
                    account_name = EXCLUDED.account_name,
                    category = EXCLUDED.category,
                    balance = EXCLUDED.balance
                "#,
            )
            .bind(period_id)
            .bind(&account.account_number)
            .bind(&account.account_name)
            .bind(&account.category)
            .bind(account.balance)
            .execute(pool)
            .await?;
        }

        let message = if validation_errors.is_empty() {
            format!(
                "Successfully parsed {} accounts for period ending {}. Period ID: {}",
                accounts.len(),
                input.period_end,
                period_id
            )
        } else {
            format!(
                "Parsed {} accounts with {} validation errors. Period ID: {}",
                accounts.len(),
                validation_errors.len(),
                period_id
            )
        };

        Ok(ParseTrialBalanceOutput {
            success: true,
            message,
            period_id: Some(period_id),
            accounts_parsed: accounts.len(),
            validation_errors,
        })
    }
}

// ============================================================================
// Tool: CalculateVariances
// ============================================================================

#[derive(Clone, Debug)]
pub struct CalculateVariances;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CalculateVariancesInput {
    /// The current period ID (from ParseTrialBalance)
    pub current_period_id: i32,
    /// The prior period ID (from ParseTrialBalance)
    pub prior_period_id: i32,
    /// Minimum variance percentage to flag as material (default: 10.0)
    pub materiality_threshold_percent: Option<f64>,
    /// Minimum absolute variance to flag as material (default: 100000.0)
    pub materiality_threshold_absolute: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculateVariancesOutput {
    pub success: bool,
    pub message: String,
    pub variance_analysis_id: Option<i32>,
    pub total_variances: usize,
    pub material_variances: usize,
    pub variances: Vec<VarianceData>,
}

#[async_trait::async_trait]
impl Tool for CalculateVariances {
    const NAME: &'static str = "calculate_variances";
    const DESCRIPTION: &'static str = r#"
    Calculate quarter-over-quarter variances between two trial balance periods.

    This tool compares accounts from the current period against the prior period and:
    1. Calculates absolute and percentage variances
    2. Flags material variances based on thresholds
    3. Assigns priority levels (High, Medium, Low)
    4. Stores the analysis in the database

    Example:
    ```json
    {
        "current_period_id": 123,
        "prior_period_id": 122,
        "materiality_threshold_percent": 10.0,
        "materiality_threshold_absolute": 100000.0
    }
    ```

    Default thresholds:
    - Material: >10% variance AND >$100K absolute change
    - High priority: >20% variance AND >$500K absolute change
    - Medium priority: >10% variance AND >$100K absolute change
    "#;

    type ToolInput = CalculateVariancesInput;
    type ToolOutput = CalculateVariancesOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let pool = &app_state.db;

        let materiality_pct = input.materiality_threshold_percent.unwrap_or(10.0);
        let materiality_abs = input.materiality_threshold_absolute.unwrap_or(100_000.0);

        // Fetch current period accounts
        let current_accounts: Vec<(String, String, f64)> = sqlx::query_as(
            r#"
            SELECT account_name, category, balance
            FROM accounts
            WHERE period_id = $1
            "#,
        )
        .bind(input.current_period_id)
        .fetch_all(pool)
        .await?;

        // Fetch prior period accounts
        let prior_accounts: Vec<(String, String, f64)> = sqlx::query_as(
            r#"
            SELECT account_name, category, balance
            FROM accounts
            WHERE period_id = $1
            "#,
        )
        .bind(input.prior_period_id)
        .fetch_all(pool)
        .await?;

        if current_accounts.is_empty() {
            return Ok(CalculateVariancesOutput {
                success: false,
                message: format!(
                    "No accounts found for current period ID {}",
                    input.current_period_id
                ),
                variance_analysis_id: None,
                total_variances: 0,
                material_variances: 0,
                variances: vec![],
            });
        }

        // Build a map of prior accounts
        let prior_map: std::collections::HashMap<String, (String, f64)> = prior_accounts
            .into_iter()
            .map(|(name, cat, bal)| (name.clone(), (cat, bal)))
            .collect();

        // Calculate variances
        let mut variances = Vec::new();
        let mut material_count = 0;

        for (account_name, category, current_balance) in current_accounts {
            let prior_balance = prior_map
                .get(&account_name)
                .map(|(_, bal)| *bal)
                .unwrap_or(0.0);

            let variance_abs = current_balance - prior_balance;
            let variance_pct = if prior_balance.abs() > 0.01 {
                (variance_abs / prior_balance.abs()) * 100.0
            } else if variance_abs.abs() > 0.01 {
                // New account or prior was zero
                100.0
            } else {
                0.0
            };

            let is_material = variance_pct.abs() > materiality_pct
                && variance_abs.abs() > materiality_abs;

            let priority = if variance_pct.abs() > 20.0 && variance_abs.abs() > 500_000.0 {
                "High"
            } else if is_material {
                "Medium"
            } else {
                "Low"
            };

            if is_material {
                material_count += 1;
            }

            // Store in database
            let _variance_id: i32 = sqlx::query_scalar(
                r#"
                INSERT INTO variance_analyses
                (current_period_id, prior_period_id, account_name, category,
                 current_balance, prior_balance, variance_absolute, variance_percent,
                 is_material, priority)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                ON CONFLICT (current_period_id, prior_period_id, account_name)
                DO UPDATE SET
                    category = EXCLUDED.category,
                    current_balance = EXCLUDED.current_balance,
                    prior_balance = EXCLUDED.prior_balance,
                    variance_absolute = EXCLUDED.variance_absolute,
                    variance_percent = EXCLUDED.variance_percent,
                    is_material = EXCLUDED.is_material,
                    priority = EXCLUDED.priority
                RETURNING id
                "#,
            )
            .bind(input.current_period_id)
            .bind(input.prior_period_id)
            .bind(&account_name)
            .bind(&category)
            .bind(current_balance)
            .bind(prior_balance)
            .bind(variance_abs)
            .bind(variance_pct)
            .bind(is_material)
            .bind(priority)
            .fetch_one(pool)
            .await?;

            variances.push(VarianceData {
                account: account_name,
                category,
                current_balance,
                prior_balance,
                variance_absolute: variance_abs,
                variance_percent: variance_pct,
                is_material,
                priority: priority.to_string(),
            });
        }

        // Sort by materiality and absolute variance
        variances.sort_by(|a, b| {
            b.is_material
                .cmp(&a.is_material)
                .then_with(|| b.variance_absolute.abs().partial_cmp(&a.variance_absolute.abs()).unwrap())
        });

        Ok(CalculateVariancesOutput {
            success: true,
            message: format!(
                "Calculated {} variances, {} are material",
                variances.len(),
                material_count
            ),
            variance_analysis_id: Some(input.current_period_id), // Use as identifier
            total_variances: variances.len(),
            material_variances: material_count,
            variances,
        })
    }
}

// ============================================================================
// Tool: GetVarianceData
// ============================================================================

#[derive(Clone, Debug)]
pub struct GetVarianceData;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetVarianceDataInput {
    /// The current period ID
    pub current_period_id: i32,
    /// The prior period ID
    pub prior_period_id: i32,
    /// Optional: fetch prior MD&A text for reference
    pub include_prior_mda: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodInfo {
    pub period_end: String,
    pub total_accounts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetVarianceDataOutput {
    pub success: bool,
    pub message: String,
    pub current_period: PeriodInfo,
    pub prior_period: PeriodInfo,
    pub variances: Vec<VarianceData>,
    pub material_variances: Vec<VarianceData>,
    pub prior_mda_text: Option<String>,
    pub variance_summary: String,
}

#[async_trait::async_trait]
impl Tool for GetVarianceData {
    const NAME: &'static str = "get_variance_data";
    const DESCRIPTION: &'static str = r#"
    Retrieve variance analysis data and period information for MD&A report generation.

    This tool fetches all the data you need to write an MD&A narrative:
    - Variance analysis results (material and all variances)
    - Period metadata (dates, account counts)
    - Optional prior quarter MD&A text for tone reference

    After calling this tool, YOU (the agent) should write the MD&A narrative yourself
    using the variance data provided. Do not call another AI to generate it.

    Example:
    ```json
    {
        "current_period_id": 102,
        "prior_period_id": 101,
        "include_prior_mda": true
    }
    ```

    The output will give you:
    - Sorted variance list (material items first)
    - Period information for both quarters
    - Summary statistics
    - Prior MD&A text (if requested and available)

    Use this data to write:
    1. Results of Operations section (300-500 words)
    2. Liquidity and Capital Resources section (200-400 words)
    3. Variance table in markdown format
    "#;

    type ToolInput = GetVarianceDataInput;
    type ToolOutput = GetVarianceDataOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let pool = &app_state.db;

        // Fetch current period info
        let (current_period_end, current_account_count): (chrono::NaiveDate, i64) =
            sqlx::query_as(
                r#"
                SELECT fp.period_end, COUNT(a.id)
                FROM financial_periods fp
                LEFT JOIN accounts a ON fp.id = a.period_id
                WHERE fp.id = $1
                GROUP BY fp.period_end
                "#,
            )
            .bind(input.current_period_id)
            .fetch_one(pool)
            .await?;

        // Fetch prior period info
        let (prior_period_end, prior_account_count): (chrono::NaiveDate, i64) = sqlx::query_as(
            r#"
            SELECT fp.period_end, COUNT(a.id)
            FROM financial_periods fp
            LEFT JOIN accounts a ON fp.id = a.period_id
            WHERE fp.id = $1
            GROUP BY fp.period_end
            "#,
        )
        .bind(input.prior_period_id)
        .fetch_one(pool)
        .await?;

        // Fetch all variances
        let variance_rows: Vec<(
            String,  // account_name
            String,  // category
            f64,     // current_balance
            f64,     // prior_balance
            f64,     // variance_absolute
            f64,     // variance_percent
            bool,    // is_material
            String,  // priority
        )> = sqlx::query_as(
            r#"
            SELECT account_name, category, current_balance, prior_balance,
                   variance_absolute, variance_percent, is_material, priority
            FROM variance_analyses
            WHERE current_period_id = $1 AND prior_period_id = $2
            ORDER BY is_material DESC, ABS(variance_absolute) DESC
            "#,
        )
        .bind(input.current_period_id)
        .bind(input.prior_period_id)
        .fetch_all(pool)
        .await?;

        let variances: Vec<VarianceData> = variance_rows
            .into_iter()
            .map(
                |(
                    account,
                    category,
                    current_balance,
                    prior_balance,
                    variance_absolute,
                    variance_percent,
                    is_material,
                    priority,
                )| VarianceData {
                    account,
                    category,
                    current_balance,
                    prior_balance,
                    variance_absolute,
                    variance_percent,
                    is_material,
                    priority,
                },
            )
            .collect();

        let material_variances: Vec<VarianceData> = variances
            .iter()
            .filter(|v| v.is_material)
            .cloned()
            .collect();

        // Optionally fetch prior MD&A text
        let prior_mda_text = if input.include_prior_mda {
            sqlx::query_scalar::<_, String>(
                r#"
                SELECT mda_text
                FROM prior_mda_context
                WHERE period_id = $1
                "#,
            )
            .bind(input.prior_period_id)
            .fetch_optional(pool)
            .await?
        } else {
            None
        };

        let variance_summary = format!(
            "Found {} total variances, {} are material (>10% and >$100K)",
            variances.len(),
            material_variances.len()
        );

        Ok(GetVarianceDataOutput {
            success: true,
            message: format!(
                "Retrieved variance data for periods {} vs {}",
                current_period_end, prior_period_end
            ),
            current_period: PeriodInfo {
                period_end: current_period_end.to_string(),
                total_accounts: current_account_count as usize,
            },
            prior_period: PeriodInfo {
                period_end: prior_period_end.to_string(),
                total_accounts: prior_account_count as usize,
            },
            variances,
            material_variances,
            prior_mda_text,
            variance_summary,
        })
    }
}

// ============================================================================
// Tool: SaveReport
// ============================================================================

#[derive(Clone, Debug)]
pub struct SaveReport;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SaveReportInput {
    /// The period ID this report is for
    pub period_id: i32,
    /// Results of Operations section text
    pub results_of_operations: String,
    /// Liquidity and Capital Resources section text
    pub liquidity_and_capital: String,
    /// Optional variance table markdown
    pub variance_table: Option<String>,
    /// Mark as finalized
    pub finalized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveReportOutput {
    pub success: bool,
    pub message: String,
    pub final_report: String,
}

#[async_trait::async_trait]
impl Tool for SaveReport {
    const NAME: &'static str = "save_report";
    const DESCRIPTION: &'static str = r#"
    Save an MD&A report that you (the agent) have written.

    After calling get_variance_data and writing the MD&A narrative yourself,
    use this tool to save the report to the database for the user.

    The report should include:
    1. Results of Operations section (300-500 words explaining material variances)
    2. Liquidity and Capital Resources section (200-400 words on cash position)
    3. Optional: variance table in markdown format

    Example:
    ```json
    {
        "period_id": 102,
        "results_of_operations": "For the three months ended September 30, 2025...",
        "liquidity_and_capital": "As of September 30, 2025, the Company had cash...",
        "variance_table": "| Account | Prior | Current | Variance $ | Variance % |\n|...",
        "finalized": true
    }
    ```
    "#;

    type ToolInput = SaveReportInput;
    type ToolOutput = SaveReportOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let pool = &app_state.db;

        let variance_table = input.variance_table.unwrap_or_default();

        let full_report = format!(
            "{}\n\n## Results of Operations\n\n{}\n\n## Liquidity and Capital Resources\n\n{}",
            variance_table, input.results_of_operations, input.liquidity_and_capital
        );

        // Insert draft
        let draft_id: i32 = sqlx::query_scalar(
            r#"
            INSERT INTO mda_drafts
            (period_id, results_of_operations, liquidity_and_capital, variance_table, full_report, thread_id, edited_by_user)
            VALUES ($1, $2, $3, $4, $5, $6, false)
            RETURNING id
            "#,
        )
        .bind(input.period_id)
        .bind(&input.results_of_operations)
        .bind(&input.liquidity_and_capital)
        .bind(&variance_table)
        .bind(&full_report)
        .bind(context.thread.thread_id)
        .fetch_one(pool)
        .await?;

        let message = if input.finalized {
            format!(
                "MD&A report saved and finalized. Draft ID: {}. Ready for export.",
                draft_id
            )
        } else {
            format!("MD&A report saved as draft. Draft ID: {}.", draft_id)
        };

        Ok(SaveReportOutput {
            success: true,
            message,
            final_report: full_report,
        })
    }
}
