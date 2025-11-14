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
// Tool: GenerateMDAReport
// ============================================================================

#[derive(Clone, Debug)]
pub struct GenerateMDAReport;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GenerateMDAReportInput {
    /// The current period ID
    pub period_id: i32,
    /// The variance analysis data (output from CalculateVariances)
    pub variances: Vec<VarianceData>,
    /// Optional prior MD&A text for tone/style reference
    pub prior_mda_text: Option<String>,
    /// Company context (e.g., "US Gold Corp", "CK Gold Project", etc.)
    pub company_context: Option<String>,
    /// Material events or notes to include
    pub material_events: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateMDAReportOutput {
    pub success: bool,
    pub message: String,
    pub draft_id: Option<i32>,
    pub full_report: String,
    pub results_of_operations: String,
    pub liquidity_and_capital: String,
    pub variance_table: String,
}

#[async_trait::async_trait]
impl Tool for GenerateMDAReport {
    const NAME: &'static str = "generate_mda_report";
    const DESCRIPTION: &'static str = r#"
    Generate an MD&A (Management Discussion & Analysis) report using AI.

    This tool takes variance analysis data and generates professional MD&A sections:
    1. Results of Operations (300-500 words)
    2. Liquidity and Capital Resources (200-400 words)
    3. Variance table in markdown format

    The AI will:
    - Use SEC-compliant language
    - Explain all material variances
    - Match the tone of prior MD&A if provided
    - Include company-specific context
    - Generate factual, professional narratives

    Example:
    ```json
    {
        "period_id": 123,
        "variances": [...],
        "company_context": "US Gold Corp (NASDAQ: USAU) - CK Gold Project development",
        "material_events": [
            "Completed feasibility study",
            "Made Final Investment Decision (FID)",
            "Began capitalizing development costs"
        ]
    }
    ```
    "#;

    type ToolInput = GenerateMDAReportInput;
    type ToolOutput = GenerateMDAReportOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        // Build variance table
        let mut variance_table = String::from("## Variance Analysis\n\n");
        variance_table.push_str("| Account | Category | Prior Period | Current Period | Variance $ | Variance % | Priority |\n");
        variance_table.push_str("|---------|----------|--------------|----------------|------------|------------|----------|\n");

        for var in &input.variances {
            if var.is_material {
                variance_table.push_str(&format!(
                    "| {} | {} | ${:.2} | ${:.2} | ${:.2} | {:.1}% | {} |\n",
                    var.account,
                    var.category,
                    var.prior_balance,
                    var.current_balance,
                    var.variance_absolute,
                    var.variance_percent,
                    var.priority
                ));
            }
        }

        // Build AI prompt
        let mut prompt = String::from("# Task\nGenerate the Management Discussion & Analysis (MD&A) sections for a quarterly SEC 10-Q filing.\n\n");

        if let Some(ctx) = &input.company_context {
            prompt.push_str(&format!("# Company Context\n{}\n\n", ctx));
        }

        prompt.push_str("# Financial Variances\n\n");
        prompt.push_str(&variance_table);
        prompt.push('\n');

        if let Some(events) = &input.material_events {
            if !events.is_empty() {
                prompt.push_str("# Material Events\n");
                for event in events {
                    prompt.push_str(&format!("- {}\n", event));
                }
                prompt.push('\n');
            }
        }

        if let Some(prior) = &input.prior_mda_text {
            prompt.push_str("# Prior Quarter MD&A (for tone/style reference)\n");
            prompt.push_str(&format!("{}\n\n", prior));
        }

        prompt.push_str(r#"# Requirements
1. Generate "Results of Operations" section (300-500 words)
2. Generate "Liquidity and Capital Resources" section (200-400 words)
3. Explain ALL material variances
4. Use professional, SEC-compliant tone
5. Be factual, concise, and avoid forward-looking statements
6. Focus on explaining WHY changes occurred, not just WHAT changed

# Output Format
Return as markdown with these exact section headers:
## Results of Operations
[content here]

## Liquidity and Capital Resources
[content here]
"#);

        // Call Claude API to generate the report
        let client = reqwest::Client::new();
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| cja::color_eyre::eyre::eyre!("ANTHROPIC_API_KEY not set"))?;

        let request_body = serde_json::json!({
            "model": "claude-sonnet-4-20250514",
            "max_tokens": 4096,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(cja::color_eyre::eyre::eyre!(
                "Claude API error: {}",
                error_text
            ));
        }

        let response_json: serde_json::Value = response.json().await?;
        let ai_content = response_json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| cja::color_eyre::eyre::eyre!("No text in Claude response"))?
            .to_string();

        // Parse the AI response to extract sections
        let results_of_operations = extract_section(&ai_content, "Results of Operations");
        let liquidity_and_capital = extract_section(&ai_content, "Liquidity and Capital Resources");

        let full_report = format!(
            "{}\n\n{}\n\n{}",
            variance_table, results_of_operations, liquidity_and_capital
        );

        // Store in database
        let pool = &app_state.db;
        let draft_id: i32 = sqlx::query_scalar(
            r#"
            INSERT INTO mda_drafts (period_id, results_of_operations, liquidity_and_capital, variance_table, full_report, thread_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
        .bind(input.period_id)
        .bind(&results_of_operations)
        .bind(&liquidity_and_capital)
        .bind(&variance_table)
        .bind(&full_report)
        .bind(context.thread.thread_id)
        .fetch_one(pool)
        .await?;

        Ok(GenerateMDAReportOutput {
            success: true,
            message: format!("Generated MD&A report. Draft ID: {}", draft_id),
            draft_id: Some(draft_id),
            full_report,
            results_of_operations,
            liquidity_and_capital,
            variance_table,
        })
    }
}

/// Helper function to extract a section from markdown
fn extract_section(content: &str, section_name: &str) -> String {
    let section_marker = format!("## {}", section_name);
    if let Some(start) = content.find(&section_marker) {
        let after_header = &content[start + section_marker.len()..];
        // Find the next ## header or end of content
        if let Some(end) = after_header.find("\n## ") {
            after_header[..end].trim().to_string()
        } else {
            after_header.trim().to_string()
        }
    } else {
        format!("[Section '{}' not found in AI response]", section_name)
    }
}

// ============================================================================
// Tool: SaveReport
// ============================================================================

#[derive(Clone, Debug)]
pub struct SaveReport;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SaveReportInput {
    /// The draft ID to save
    pub draft_id: i32,
    /// Optional edited text for Results of Operations
    pub edited_results_of_operations: Option<String>,
    /// Optional edited text for Liquidity and Capital
    pub edited_liquidity_and_capital: Option<String>,
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
    Save or update an MD&A report draft.

    This tool allows you to:
    1. Save user edits to the generated report
    2. Mark the report as finalized
    3. Track changes for future learning

    Example:
    ```json
    {
        "draft_id": 123,
        "edited_results_of_operations": "Updated text...",
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
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let pool = &app_state.db;

        // Fetch current draft
        let (orig_results, orig_liquidity, variance_table): (String, String, String) =
            sqlx::query_as(
                r#"
                SELECT results_of_operations, liquidity_and_capital, variance_table
                FROM mda_drafts
                WHERE id = $1
                "#,
            )
            .bind(input.draft_id)
            .fetch_one(pool)
            .await?;

        let mut edited = false;

        // Update if edited
        if let Some(edited_results) = &input.edited_results_of_operations {
            if edited_results != &orig_results {
                sqlx::query(
                    r#"
                    INSERT INTO user_edits (draft_id, section, original_text, edited_text)
                    VALUES ($1, 'results_of_operations', $2, $3)
                    "#,
                )
                .bind(input.draft_id)
                .bind(&orig_results)
                .bind(edited_results)
                .execute(pool)
                .await?;

                edited = true;
            }
        }

        if let Some(edited_liquidity) = &input.edited_liquidity_and_capital {
            if edited_liquidity != &orig_liquidity {
                sqlx::query(
                    r#"
                    INSERT INTO user_edits (draft_id, section, original_text, edited_text)
                    VALUES ($1, 'liquidity_and_capital', $2, $3)
                    "#,
                )
                .bind(input.draft_id)
                .bind(&orig_liquidity)
                .bind(edited_liquidity)
                .execute(pool)
                .await?;

                edited = true;
            }
        }

        let final_results = input
            .edited_results_of_operations
            .as_ref()
            .unwrap_or(&orig_results);
        let final_liquidity = input
            .edited_liquidity_and_capital
            .as_ref()
            .unwrap_or(&orig_liquidity);

        let final_report = format!(
            "{}\n\n## Results of Operations\n\n{}\n\n## Liquidity and Capital Resources\n\n{}",
            variance_table, final_results, final_liquidity
        );

        // Update draft
        sqlx::query(
            r#"
            UPDATE mda_drafts
            SET results_of_operations = $1,
                liquidity_and_capital = $2,
                full_report = $3,
                edited_by_user = $4
            WHERE id = $5
            "#,
        )
        .bind(final_results)
        .bind(final_liquidity)
        .bind(&final_report)
        .bind(edited)
        .bind(input.draft_id)
        .execute(pool)
        .await?;

        let message = if input.finalized {
            format!("Report finalized and saved. Draft ID: {}", input.draft_id)
        } else if edited {
            format!("Report updated with edits. Draft ID: {}", input.draft_id)
        } else {
            format!("Report saved. Draft ID: {}", input.draft_id)
        };

        Ok(SaveReportOutput {
            success: true,
            message,
            final_report,
        })
    }
}
