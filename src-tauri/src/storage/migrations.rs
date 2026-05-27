pub fn get_migrations() -> Vec<&'static str> {
    vec![
        // Tasks Table
        r#"CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            user_request TEXT NOT NULL,
            status TEXT NOT NULL,
            planning_status TEXT NOT NULL,
            execution_status TEXT NOT NULL,
            current_gate TEXT,
            last_valid_state_id TEXT,
            risk_level TEXT NOT NULL,
            approval_status TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );"#,

        // Task Breakdown Table
        r#"CREATE TABLE IF NOT EXISTS task_breakdown (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            parent_id TEXT,
            level INTEGER NOT NULL,
            topic TEXT NOT NULL,
            subtopic TEXT NOT NULL,
            criterion TEXT NOT NULL,
            subcriterion TEXT NOT NULL,
            description TEXT,
            risk_pre_label TEXT,
            probable_connector TEXT,
            decision_node_required TEXT,
            FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );"#,

        // Decision Nodes Table
        r#"CREATE TABLE IF NOT EXISTS decision_nodes (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            breakdown_id TEXT NOT NULL,
            level INTEGER NOT NULL,
            parent_node_id TEXT,
            required_approval INTEGER DEFAULT 0,
            gate_status TEXT NOT NULL,
            authorized_decider_type TEXT NOT NULL,
            authorized_decider_id TEXT NOT NULL,
            status TEXT NOT NULL,
            selected_option TEXT,
            reason TEXT,
            evidence_json TEXT,
            confidence REAL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE,
            FOREIGN KEY(breakdown_id) REFERENCES task_breakdown(id) ON DELETE CASCADE
        );"#,

        // Statements Table
        r#"CREATE TABLE IF NOT EXISTS statements (
            id TEXT PRIMARY KEY,
            decision_node_id TEXT NOT NULL,
            source_type TEXT NOT NULL,
            source_name TEXT NOT NULL,
            content TEXT NOT NULL,
            evidence_ref TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(decision_node_id) REFERENCES decision_nodes(id) ON DELETE CASCADE
        );"#,

        // Alternatives Table
        r#"CREATE TABLE IF NOT EXISTS alternatives (
            id TEXT PRIMARY KEY,
            decision_node_id TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            pros_json TEXT NOT NULL,
            cons_json TEXT NOT NULL,
            accuracy_score INTEGER NOT NULL,
            safety_score INTEGER NOT NULL,
            dependency_score INTEGER NOT NULL,
            rollback_score INTEGER NOT NULL,
            maintainability_score INTEGER NOT NULL,
            cost_score INTEGER NOT NULL,
            time_score INTEGER NOT NULL,
            user_control_score INTEGER NOT NULL,
            live_impact_score INTEGER NOT NULL,
            data_loss_risk_score INTEGER NOT NULL,
            selected INTEGER DEFAULT 0,
            reason TEXT,
            FOREIGN KEY(decision_node_id) REFERENCES decision_nodes(id) ON DELETE CASCADE
        );"#,

        // Risk Assessments Table
        r#"CREATE TABLE IF NOT EXISTS risk_assessments (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            decision_node_id TEXT NOT NULL,
            risk_level TEXT NOT NULL,
            risk_reason TEXT NOT NULL,
            affected_assets_json TEXT NOT NULL,
            mitigation_plan TEXT NOT NULL,
            FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE,
            FOREIGN KEY(decision_node_id) REFERENCES decision_nodes(id) ON DELETE CASCADE
        );"#,

        // Checkpoints Table
        r#"CREATE TABLE IF NOT EXISTS checkpoints (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            decision_node_id TEXT,
            checkpoint_type TEXT NOT NULL,
            status TEXT NOT NULL,
            result TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );"#,

        // Tests Table
        r#"CREATE TABLE IF NOT EXISTS tests (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            test_name TEXT NOT NULL,
            expected_result TEXT NOT NULL,
            actual_result TEXT,
            status TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );"#,

        // Approvals Table
        r#"CREATE TABLE IF NOT EXISTS approvals (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            decision_node_id TEXT,
            approver_id TEXT,
            approver_role TEXT,
            approval_source TEXT,
            action TEXT NOT NULL,
            risk_level TEXT NOT NULL,
            requested_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            approved_at DATETIME,
            status TEXT NOT NULL,
            user_note TEXT,
            FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );"#,

        // Execution Logs Table
        r#"CREATE TABLE IF NOT EXISTS execution_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            level TEXT NOT NULL,
            message TEXT NOT NULL,
            gate_name TEXT,
            event_type TEXT,
            metadata_json TEXT,
            FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );"#,

        // Snapshots Table
        r#"CREATE TABLE IF NOT EXISTS snapshots (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            target_type TEXT NOT NULL,
            target_path TEXT NOT NULL,
            snapshot_path TEXT NOT NULL,
            hash_before TEXT,
            hash_after TEXT,
            state_id TEXT,
            rollback_status TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );"#,

        // State History Table
        r#"CREATE TABLE IF NOT EXISTS state_history (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            state_name TEXT NOT NULL,
            state_json TEXT NOT NULL,
            is_valid INTEGER NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );"#,

        // Reports Table
        r#"CREATE TABLE IF NOT EXISTS reports (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            report_type TEXT NOT NULL,
            content TEXT NOT NULL,
            file_path TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );"#,

        // Dependency Assessments Table
        r#"CREATE TABLE IF NOT EXISTS dependency_assessments (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            target_id TEXT NOT NULL,
            target_type TEXT NOT NULL,
            dependency_level TEXT NOT NULL,
            status TEXT NOT NULL,
            reason TEXT NOT NULL,
            network_required INTEGER NOT NULL,
            api_key_required INTEGER NOT NULL,
            live_system INTEGER NOT NULL,
            approval_required INTEGER NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );"#,

        r#"CREATE INDEX IF NOT EXISTS idx_approvals_task_node_action_risk_status
            ON approvals(task_id, decision_node_id, action, risk_level, status);"#,

        r#"CREATE INDEX IF NOT EXISTS idx_approvals_authorized_signatures
            ON approvals(task_id, decision_node_id, action, risk_level, approver_id, approver_role, status);"#
    ]
}
