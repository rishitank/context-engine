/**
 * Reactive AI Code Review Engine - Type Definitions
 *
 * Phase 1: Type extensions for reactive mode capabilities
 *
 * These types extend the existing planning types to support:
 * - Commit-keyed caching
 * - Parallel execution metadata
 * - Reactive review sessions
 */

import { EnhancedPlanOutput, EnhancedPlanStep } from '../mcp/types/planning.js';

// ============================================================================
// Cache Configuration Types
// ============================================================================

/**
 * Options for commit-based cache keying
 */
export interface CommitCacheOptions {
    /** Enable commit hash as cache key prefix */
    enable_commit_keying: boolean;

    /** Enable background prefetching of file context */
    enable_prefetch: boolean;

    /** How many levels of dependencies to prefetch (default: 2) */
    prefetch_depth: number;

    /** Cache TTL in milliseconds (default: 300000 = 5 minutes) */
    cache_ttl_ms?: number;
}

/**
 * Cache statistics for telemetry
 */
export interface CacheStats {
    /** Number of entries in cache */
    size: number;

    /** Cache hit rate (0-1) */
    hitRate: number;

    /** Whether commit-keyed caching is enabled */
    commitKeyed: boolean;

    /** Current commit hash (if commit-keyed) */
    currentCommit: string | null;

    /** Total cache hits */
    hits?: number;

    /** Total cache misses */
    misses?: number;
}

// ============================================================================
// Parallel Execution Types
// ============================================================================

/**
 * Options for parallel step execution
 */
export interface ParallelExecutionOptions {
    /** Maximum concurrent workers (default: 3) */
    max_workers: number;

    /** Enable parallel execution mode */
    enable_parallel: boolean;

    /** Timeout per node in milliseconds (default: 60000) */
    node_timeout_ms: number;

    /** Maximum retries per step (default: 2) */
    max_retries?: number;
}

/**
 * Execution metadata for a single step in reactive mode
 */
export interface ReactiveStepExtension {
    /** Execution model configuration */
    execution_model?: {
        /** Whether this step can run in parallel with others */
        parallel_safe: boolean;

        /** Execution priority (0 = highest) */
        priority: number;

        /** Step-specific timeout in milliseconds */
        timeout_ms: number;

        /** Number of retry attempts */
        retry_count: number;
    };

    /** Context metadata for this step */
    context_metadata?: {
        /** Commit hash for cache consistency */
        commit_hash: string;

        /** Token budget allocated to this step */
        token_budget: number;

        /** Whether context was served from cache */
        cache_hit?: boolean;

        /** Parallel group this step belongs to */
        parallel_group?: number;
    };
}

// ============================================================================
// Reactive Plan Types
// ============================================================================

/**
 * Metadata for a PR under reactive review
 */
export interface PRMetadata {
    /** PR number */
    pr_number?: number;

    /** Base branch/ref (e.g., "main") */
    base_ref: string;

    /** Head commit hash */
    commit_hash: string;

    /** Changed file paths */
    changed_files: string[];

    /** Total lines added */
    lines_added?: number;

    /** Total lines removed */
    lines_removed?: number;

    /** PR title */
    title?: string;

    /** PR author */
    author?: string;
}

/**
 * Extended plan output for reactive mode
 */
export interface ReactivePlan extends EnhancedPlanOutput {
    /** Flag indicating this is a reactive plan */
    reactive_mode: true;

    /** PR and context metadata */
    context_metadata: {
        /** Git commit hash for cache keying */
        commit_hash: string;

        /** Base reference (branch or commit) */
        base_ref: string;

        /** Total token budget for this review */
        token_budget: number;

        /** PR number if applicable */
        pr_number?: number;

        /** Diff content (truncated) */
        diff_summary?: string;
    };
}

// ============================================================================
// Review Session Types
// ============================================================================

/**
 * Status of a reactive review session
 */
export type ReviewSessionStatus =
    | 'initializing'
    | 'analyzing'
    | 'executing'
    | 'paused'
    | 'completed'
    | 'failed'
    | 'cancelled';

/**
 * A reactive review session
 */
export interface ReviewSession {
    /** Unique session ID */
    session_id: string;

    /** Associated plan ID */
    plan_id: string;

    /** Current session status */
    status: ReviewSessionStatus;

    /** PR metadata */
    pr_metadata: PRMetadata;

    /** Session creation time */
    created_at: string;

    /** Last update time */
    updated_at: string;

    /** Current step being executed */
    current_step?: number;

    /** Total steps in the review */
    total_steps?: number;

    /** Error message if failed */
    error?: string;

    /** HITL checkpoint ID if paused */
    checkpoint_id?: string;
}

/**
 * Detailed status of a review session
 */
export interface ReviewStatus {
    /** The session */
    session: ReviewSession;

    /** Execution progress */
    progress: {
        completed_steps: number;
        total_steps: number;
        percentage: number;
    };

    /** Telemetry data */
    telemetry?: {
        start_time: string;
        elapsed_ms: number;
        tokens_used: number;
        cache_hit_rate: number;
        /** Milliseconds since last activity (for zombie detection) */
        last_activity_ms?: number;
        /** True if session appears stalled (no activity for 2+ minutes) */
        appears_stalled?: boolean;
    };

    /** Findings so far */
    findings_count?: number;
}

// ============================================================================
// Validation Types (Phase 4 preview)
// ============================================================================

/**
 * Validation tier
 */
export type ValidationTier = 'deterministic' | 'heuristic' | 'llm';

/**
 * Severity level for validation findings
 */
export type ValidationSeverity = 'error' | 'warning' | 'info';

/**
 * A finding from the validation pipeline
 */
export interface ValidationFinding {
    /** Unique finding ID */
    id: string;

    /** Which validation tier found this */
    tier: ValidationTier;

    /** Type of finding (e.g., "syntax_error", "secret_detected") */
    type: string;

    /** Human-readable message */
    message: string;

    /** Severity level */
    severity: ValidationSeverity;

    /** File path if applicable */
    file_path?: string;

    /** Line number if applicable */
    line?: number;

    /** Suggested fix if available */
    suggestion?: string;
}

// ============================================================================
// Telemetry Types
// ============================================================================

/**
 * Reactive-specific telemetry event types
 */
export type ReactiveEventType =
    | 'reactive.cache.hit'
    | 'reactive.cache.miss'
    | 'reactive.cache.invalidate'
    | 'reactive.parallel.started'
    | 'reactive.parallel.completed'
    | 'reactive.step.started'
    | 'reactive.step.completed'
    | 'reactive.step.failed'
    | 'reactive.validation.tier1'
    | 'reactive.validation.tier2'
    | 'reactive.validation.tier3'
    | 'reactive.guardrail.secret_detected'
    | 'reactive.guardrail.token_limit'
    | 'reactive.checkpoint.created'
    | 'reactive.checkpoint.resolved';

/**
 * Telemetry event data
 */
export interface ReactiveEvent {
    /** Event type */
    type: ReactiveEventType;

    /** Timestamp */
    timestamp: string;

    /** Session ID */
    session_id?: string;

    /** Plan ID */
    plan_id?: string;

    /** Step number */
    step_number?: number;

    /** Duration in milliseconds */
    duration_ms?: number;

    /** Additional data */
    data?: Record<string, unknown>;
}
