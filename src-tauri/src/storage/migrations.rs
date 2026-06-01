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
        // Alternatives generated for every atomic task part before execution.
        r#"CREATE TABLE IF NOT EXISTS task_breakdown_alternatives (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            breakdown_id TEXT NOT NULL,
            alternative_order INTEGER NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            accepted_correct INTEGER NOT NULL DEFAULT 0,
            selected_best INTEGER NOT NULL DEFAULT 0,
            selection_reason TEXT NOT NULL,
            control_criteria TEXT NOT NULL,
            test_criteria TEXT NOT NULL,
            rollback_note TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE,
            FOREIGN KEY(breakdown_id) REFERENCES task_breakdown(id) ON DELETE CASCADE
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
            real_world_basis TEXT,
            testability_score INTEGER DEFAULT 0,
            ethical_safety_note TEXT,
            selection_reason TEXT,
            accepted_correct_approach_reason TEXT,
            selected_best_option_reason TEXT,
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
        // Operation Audit Events Table
        r#"CREATE TABLE IF NOT EXISTS operation_audit_events (
            id TEXT PRIMARY KEY,
            actor TEXT NOT NULL,
            action TEXT NOT NULL,
            target_type TEXT,
            target_id TEXT,
            status TEXT NOT NULL CHECK(status IN ('PASS', 'FAIL', 'WARN')),
            details TEXT,
            metadata_json TEXT,
            error_message TEXT,
            correlation_id TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
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
        r#"CREATE TABLE IF NOT EXISTS operation_steps (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            step_order INTEGER NOT NULL,
            expected_action TEXT NOT NULL,
            description TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );"#,
        r#"CREATE TABLE IF NOT EXISTS operation_packages (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            package_order INTEGER NOT NULL,
            package_type TEXT NOT NULL,
            subject TEXT NOT NULL,
            sub_topic TEXT NOT NULL,
            criterion TEXT NOT NULL,
            sub_criterion TEXT NOT NULL,
            accepted_truth TEXT NOT NULL,
            selected_best_alternative TEXT NOT NULL,
            operation_sequence TEXT NOT NULL,
            technology TEXT NOT NULL,
            impact_area TEXT NOT NULL,
            control_point TEXT NOT NULL,
            control_criteria TEXT NOT NULL,
            test_plan TEXT NOT NULL,
            rollback_plan TEXT NOT NULL,
            executor_role TEXT NOT NULL,
            correctness_guard_role TEXT NOT NULL,
            controller_role TEXT NOT NULL,
            independent_verifier_role TEXT NOT NULL,
            final_approver_role TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );"#,
        r#"CREATE TABLE IF NOT EXISTS operation_monitor_logs (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            decision_node_id TEXT,
            expected_action TEXT,
            actual_action TEXT NOT NULL,
            gate_name TEXT,
            status TEXT NOT NULL,
            message TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );"#,
        r#"CREATE TABLE IF NOT EXISTS principle_evaluations (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            decision_node_id TEXT,
            accepted_correct_approach_reason TEXT NOT NULL,
            selected_best_option_reason TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );"#,
        r#"CREATE TABLE IF NOT EXISTS ai_tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            risk_level TEXT NOT NULL CHECK(risk_level IN ('low', 'medium', 'high', 'critical')),
            status TEXT NOT NULL CHECK(status IN ('pending', 'approved', 'in_progress', 'completed', 'failed', 'rejected')),
            created_by TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );"#,
        r#"CREATE TABLE IF NOT EXISTS ai_task_allocations (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            platform_name TEXT NOT NULL CHECK(platform_name IN ('codex', 'open_agent_manager', 'antigravity', 'cursor', 'perplexity', 'verdent')),
            assigned_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            status TEXT NOT NULL CHECK(status IN ('waiting', 'processing', 'submitted', 'failed', 'rejected')),
            payload_file_path TEXT NOT NULL,
            FOREIGN KEY(task_id) REFERENCES ai_tasks(id) ON DELETE CASCADE,
            UNIQUE(task_id, platform_name)
        );"#,
        r#"CREATE TABLE IF NOT EXISTS ai_collected_reports (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            platform_name TEXT NOT NULL CHECK(platform_name IN ('codex', 'open_agent_manager', 'antigravity', 'cursor', 'perplexity', 'verdent')),
            submitted_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            report_path TEXT NOT NULL,
            is_verified INTEGER DEFAULT 0 CHECK(is_verified IN (0, 1)),
            verification_error TEXT,
            FOREIGN KEY(task_id) REFERENCES ai_tasks(id) ON DELETE CASCADE,
            UNIQUE(task_id, platform_name)
        );"#,
        r#"CREATE INDEX IF NOT EXISTS idx_approvals_task_node_action_risk_status
            ON approvals(task_id, decision_node_id, action, risk_level, status);"#,
        r#"CREATE INDEX IF NOT EXISTS idx_approvals_authorized_signatures
            ON approvals(task_id, decision_node_id, action, risk_level, approver_id, approver_role, status);"#,
        r#"CREATE INDEX IF NOT EXISTS idx_operation_audit_events_created_at
            ON operation_audit_events(created_at DESC);"#,
    ]
}

#[cfg(test)]
mod tests {
    use super::get_migrations;
    use rusqlite::Connection;

    fn setup_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();

        for migration in get_migrations() {
            conn.execute(migration, []).unwrap();
        }

        conn
    }

    fn insert_valid_ai_task(conn: &Connection, task_id: &str) {
        conn.execute(
            "INSERT INTO ai_tasks (id, title, description, risk_level, status, created_by)
             VALUES (?1, 'AI Task', 'Schema integration test', 'high', 'pending', 'codex')",
            [task_id],
        )
        .unwrap();
    }

    #[test]
    fn valid_ai_task_insert_succeeds() {
        let conn = setup_conn();

        insert_valid_ai_task(&conn, "ai_task_valid");

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM ai_tasks WHERE id = 'ai_task_valid'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn invalid_ai_task_risk_level_fails() {
        let conn = setup_conn();

        let result = conn.execute(
            "INSERT INTO ai_tasks (id, title, risk_level, status, created_by)
             VALUES ('ai_task_bad_risk', 'Bad Risk', 'severe', 'pending', 'codex')",
            [],
        );

        assert!(result.is_err());
    }

    #[test]
    fn invalid_ai_task_status_fails() {
        let conn = setup_conn();

        let result = conn.execute(
            "INSERT INTO ai_tasks (id, title, risk_level, status, created_by)
             VALUES ('ai_task_bad_status', 'Bad Status', 'low', 'queued', 'codex')",
            [],
        );

        assert!(result.is_err());
    }

    #[test]
    fn invalid_platform_name_fails() {
        let conn = setup_conn();
        insert_valid_ai_task(&conn, "ai_task_platform");

        let result = conn.execute(
            "INSERT INTO ai_task_allocations (id, task_id, platform_name, status, payload_file_path)
             VALUES ('alloc_bad_platform', 'ai_task_platform', 'unknown_platform', 'waiting', 'ai_workflow/tasks/task.json')",
            [],
        );

        assert!(result.is_err());
    }

    #[test]
    fn duplicate_task_platform_allocation_fails() {
        let conn = setup_conn();
        insert_valid_ai_task(&conn, "ai_task_alloc_duplicate");

        conn.execute(
            "INSERT INTO ai_task_allocations (id, task_id, platform_name, status, payload_file_path)
             VALUES ('alloc_one', 'ai_task_alloc_duplicate', 'codex', 'waiting', 'ai_workflow/tasks/task.json')",
            [],
        )
        .unwrap();
        let result = conn.execute(
            "INSERT INTO ai_task_allocations (id, task_id, platform_name, status, payload_file_path)
             VALUES ('alloc_two', 'ai_task_alloc_duplicate', 'codex', 'waiting', 'ai_workflow/tasks/task-2.json')",
            [],
        );

        assert!(result.is_err());
    }

    #[test]
    fn duplicate_task_platform_report_fails() {
        let conn = setup_conn();
        insert_valid_ai_task(&conn, "ai_task_report_duplicate");

        conn.execute(
            "INSERT INTO ai_collected_reports (id, task_id, platform_name, report_path, is_verified)
             VALUES ('report_one', 'ai_task_report_duplicate', 'codex', 'ai_workflow/collected_reports/report.md', 0)",
            [],
        )
        .unwrap();
        let result = conn.execute(
            "INSERT INTO ai_collected_reports (id, task_id, platform_name, report_path, is_verified)
             VALUES ('report_two', 'ai_task_report_duplicate', 'codex', 'ai_workflow/collected_reports/report-2.md', 1)",
            [],
        );

        assert!(result.is_err());
    }

    #[test]
    fn deleting_task_cascades_allocation_and_report() {
        let conn = setup_conn();
        insert_valid_ai_task(&conn, "ai_task_cascade");

        conn.execute(
            "INSERT INTO ai_task_allocations (id, task_id, platform_name, status, payload_file_path)
             VALUES ('alloc_cascade', 'ai_task_cascade', 'codex', 'waiting', 'ai_workflow/tasks/task.json')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ai_collected_reports (id, task_id, platform_name, report_path, is_verified)
             VALUES ('report_cascade', 'ai_task_cascade', 'codex', 'ai_workflow/collected_reports/report.md', 1)",
            [],
        )
        .unwrap();

        conn.execute("DELETE FROM ai_tasks WHERE id = 'ai_task_cascade'", [])
            .unwrap();

        let allocation_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM ai_task_allocations WHERE task_id = 'ai_task_cascade'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let report_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM ai_collected_reports WHERE task_id = 'ai_task_cascade'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(allocation_count, 0);
        assert_eq!(report_count, 0);
    }

    #[test]
    fn allocation_payload_file_path_not_null() {
        let conn = setup_conn();
        insert_valid_ai_task(&conn, "ai_task_payload_required");

        let result = conn.execute(
            "INSERT INTO ai_task_allocations (id, task_id, platform_name, status)
             VALUES ('alloc_missing_payload', 'ai_task_payload_required', 'codex', 'waiting')",
            [],
        );

        assert!(result.is_err());
    }

    #[test]
    fn report_is_verified_accepts_only_zero_or_one() {
        let conn = setup_conn();
        insert_valid_ai_task(&conn, "ai_task_verified");

        conn.execute(
            "INSERT INTO ai_collected_reports (id, task_id, platform_name, report_path, is_verified)
             VALUES ('report_verified_zero', 'ai_task_verified', 'codex', 'ai_workflow/collected_reports/report.md', 0)",
            [],
        )
        .unwrap();
        let result = conn.execute(
            "INSERT INTO ai_collected_reports (id, task_id, platform_name, report_path, is_verified)
             VALUES ('report_verified_bad', 'ai_task_verified', 'cursor', 'ai_workflow/collected_reports/report-2.md', 2)",
            [],
        );

        assert!(result.is_err());
    }

    #[test]
    fn ai_task_created_by_not_null() {
        let conn = setup_conn();

        let result = conn.execute(
            "INSERT INTO ai_tasks (id, title, risk_level, status)
             VALUES ('ai_task_missing_creator', 'Missing Creator', 'low', 'pending')",
            [],
        );

        assert!(result.is_err());
    }
}
