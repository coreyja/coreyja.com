-- Create tables for SEC reporting functionality

-- Financial periods represent a reporting period (typically a quarter)
CREATE TABLE financial_periods (
    id SERIAL PRIMARY KEY,
    period_end DATE NOT NULL,
    company_id INT NOT NULL DEFAULT 1, -- For future multi-company support
    uploaded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    thread_id INT REFERENCES agentic_threads(id) ON DELETE SET NULL,
    UNIQUE(period_end, company_id)
);

CREATE INDEX idx_financial_periods_period_end ON financial_periods(period_end);
CREATE INDEX idx_financial_periods_thread_id ON financial_periods(thread_id);

-- Accounts represent individual line items from trial balance
CREATE TABLE accounts (
    id SERIAL PRIMARY KEY,
    period_id INT NOT NULL REFERENCES financial_periods(id) ON DELETE CASCADE,
    account_number VARCHAR(50) NOT NULL,
    account_name TEXT NOT NULL,
    category VARCHAR(50) NOT NULL,
    balance DECIMAL(15,2) NOT NULL,
    UNIQUE(period_id, account_number)
);

CREATE INDEX idx_accounts_period_id ON accounts(period_id);
CREATE INDEX idx_accounts_category ON accounts(category);

-- Variance analysis results
CREATE TABLE variance_analyses (
    id SERIAL PRIMARY KEY,
    current_period_id INT NOT NULL REFERENCES financial_periods(id) ON DELETE CASCADE,
    prior_period_id INT NOT NULL REFERENCES financial_periods(id) ON DELETE CASCADE,
    account_name TEXT NOT NULL,
    category VARCHAR(50) NOT NULL,
    current_balance DECIMAL(15,2) NOT NULL,
    prior_balance DECIMAL(15,2) NOT NULL,
    variance_absolute DECIMAL(15,2) NOT NULL,
    variance_percent DECIMAL(10,4) NOT NULL,
    is_material BOOLEAN NOT NULL DEFAULT FALSE,
    priority VARCHAR(20) NOT NULL, -- 'High', 'Medium', 'Low'
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(current_period_id, prior_period_id, account_name)
);

CREATE INDEX idx_variance_current_period ON variance_analyses(current_period_id);
CREATE INDEX idx_variance_is_material ON variance_analyses(is_material);

-- Generated MD&A drafts
CREATE TABLE mda_drafts (
    id SERIAL PRIMARY KEY,
    period_id INT NOT NULL REFERENCES financial_periods(id) ON DELETE CASCADE,
    variance_analysis_id INT REFERENCES variance_analyses(id) ON DELETE SET NULL,
    results_of_operations TEXT,
    liquidity_and_capital TEXT,
    variance_table TEXT,
    full_report TEXT, -- Combined markdown report
    generated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    edited_by_user BOOLEAN DEFAULT FALSE,
    thread_id INT REFERENCES agentic_threads(id) ON DELETE SET NULL
);

CREATE INDEX idx_mda_drafts_period_id ON mda_drafts(period_id);
CREATE INDEX idx_mda_drafts_thread_id ON mda_drafts(thread_id);

-- User edits for learning and improvement
CREATE TABLE user_edits (
    id SERIAL PRIMARY KEY,
    draft_id INT NOT NULL REFERENCES mda_drafts(id) ON DELETE CASCADE,
    section VARCHAR(50) NOT NULL, -- 'results_of_operations', 'liquidity_and_capital', etc.
    original_text TEXT,
    edited_text TEXT NOT NULL,
    edited_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_user_edits_draft_id ON user_edits(draft_id);

-- Prior MD&A text for context (optional reference)
CREATE TABLE prior_mda_context (
    id SERIAL PRIMARY KEY,
    period_id INT NOT NULL REFERENCES financial_periods(id) ON DELETE CASCADE,
    mda_text TEXT NOT NULL,
    uploaded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(period_id)
);
