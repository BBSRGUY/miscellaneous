"""
Command-line interface for Ganit-A.

Beautiful terminal interface using Click and Rich.
"""

from __future__ import annotations

import json
import logging
import sys
from pathlib import Path
from typing import Any

import click
import yaml
from rich.console import Console
from rich.logging import RichHandler
from rich.panel import Panel
from rich.progress import Progress, SpinnerColumn, TextColumn
from rich.table import Table
from rich.text import Text

from ganita import __version__
from ganita.auto_discovery import AutoDiscovery
from ganita.checkpoint import ExplorationCheckpoint
from ganita.models import ModuleConfig, RunConfig
from ganita.orchestrator import run_exploration
from ganita.storage.db import Database

console = Console()


def setup_logging(verbose: bool = False) -> None:
    """Set up logging with Rich handler."""
    level = logging.DEBUG if verbose else logging.INFO

    logging.basicConfig(
        level=level,
        format="%(message)s",
        handlers=[RichHandler(rich_tracebacks=True, console=console)],
    )


@click.group()
@click.version_option(version=__version__)
@click.option("-v", "--verbose", is_flag=True, help="Enable verbose logging")
@click.pass_context
def cli(ctx: click.Context, verbose: bool) -> None:
    """
    Ganit-A: Advanced Mathematical Discovery Engine

    Discover novel mathematical constants and patterns using rigorous
    interval arithmetic and PSLQ recognition.
    """
    setup_logging(verbose)
    ctx.ensure_object(dict)
    ctx.obj["verbose"] = verbose


@cli.command()
@click.option(
    "--config",
    "-c",
    type=click.Path(exists=True, path_type=Path),
    default="config.yaml",
    help="Configuration file path",
)
@click.option(
    "--db",
    type=click.Path(path_type=Path),
    default="ganita_data/math_discovery.db",
    help="Database path",
)
@click.option(
    "--artifacts",
    type=click.Path(path_type=Path),
    default="ganita_data/artifacts",
    help="Artifacts directory",
)
def run(config: Path, db: Path, artifacts: Path) -> None:
    """Run a full mathematical discovery exploration."""

    console.print(
        Panel.fit(
            "[bold cyan]Ganit-A Mathematical Discovery Engine[/bold cyan]\n"
            f"[dim]Version {__version__}[/dim]",
            border_style="cyan",
        )
    )

    # Load configuration
    if config.exists():
        with open(config) as f:
            config_dict = yaml.safe_load(f)
        run_config = RunConfig.model_validate(config_dict)
    else:
        console.print(f"[yellow]Config file not found, using defaults[/yellow]")
        run_config = _create_default_config()

    console.print(f"\n[bold]Configuration:[/bold]")
    console.print(f"  Target precision: {run_config.target_precision_digits} digits")
    console.print(f"  Max workers: {run_config.max_workers}")
    console.print(f"  Time limit: {run_config.wallclock_limit_minutes} minutes")
    console.print(f"  Database: {db}")
    console.print(f"  Artifacts: {artifacts}\n")

    # Run exploration
    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        console=console,
    ) as progress:
        task = progress.add_task("[cyan]Running discovery pipeline...", total=None)

        try:
            run_id = run_exploration(run_config, db, artifacts)
            progress.update(task, completed=True)

            console.print(f"\n[bold green]✓[/bold green] Run completed: ID {run_id}")

            # Show summary
            _show_run_summary(db, run_id)

        except Exception as e:
            console.print(f"\n[bold red]✗ Error:[/bold red] {e}")
            if ctx := click.get_current_context(silent=True):
                if ctx.obj and ctx.obj.get("verbose"):
                    console.print_exception()
            sys.exit(1)


@cli.command()
@click.option("--db", type=click.Path(exists=True, path_type=Path), default="ganita_data/math_discovery.db")
@click.option("--run-id", type=int, default=None, help="Specific run ID")
@click.option("--limit", "-n", type=int, default=20, help="Number of results")
@click.option(
    "--stage",
    type=click.Choice(["observed", "certified", "recognized", "cross_validated"]),
    help="Filter by stage",
)
def top(db: Path, run_id: int | None, limit: int, stage: str | None) -> None:
    """Show top discoveries by rank."""

    if not db.exists():
        console.print(f"[red]Database not found: {db}[/red]")
        sys.exit(1)

    database = Database(db)

    discoveries = database.query_top(stage=stage, limit=limit, run_id=run_id)

    if not discoveries:
        console.print("[yellow]No discoveries found[/yellow]")
        return

    # Create table
    table = Table(title=f"Top {limit} Discoveries", border_style="cyan")
    table.add_column("ID", style="cyan", justify="right")
    table.add_column("Value", style="white", max_width=30)
    table.add_column("Method", style="blue")
    table.add_column("Stage", style="green")
    table.add_column("Score", style="yellow", justify="right")
    table.add_column("Precision", style="magenta", justify="right")

    for disc in discoveries:
        value_str = disc["value_repr"][:30] + "..." if len(disc["value_repr"]) > 30 else disc["value_repr"]

        stage_text = Text(disc["stage"], style=_get_stage_style(disc["stage"]))

        table.add_row(
            str(disc["discovery_id"]),
            value_str,
            disc["module_name"],
            stage_text,
            f"{disc['rank_score']:.4f}",
            f"{disc['ulp_bits']} bits",
        )

    console.print(table)


@cli.command()
@click.argument("discovery_id", type=int)
@click.option("--db", type=click.Path(exists=True, path_type=Path), default="ganita_data/math_discovery.db")
def show(discovery_id: int, db: Path) -> None:
    """Show detailed information about a discovery."""

    if not db.exists():
        console.print(f"[red]Database not found: {db}[/red]")
        sys.exit(1)

    database = Database(db)
    disc = database.get_discovery(discovery_id)

    if not disc:
        console.print(f"[red]Discovery {discovery_id} not found[/red]")
        sys.exit(1)

    # Show discovery details
    console.print(Panel.fit(f"[bold cyan]Discovery #{discovery_id}[/bold cyan]", border_style="cyan"))

    console.print(f"\n[bold]Value:[/bold]")
    console.print(f"  {disc['value_repr']}")

    console.print(f"\n[bold]Metadata:[/bold]")
    console.print(f"  Method: {disc['module_name']}")
    console.print(f"  Stage: {disc['stage']}")
    console.print(f"  Rank Score: {disc['rank_score']:.6f}")
    console.print(f"  Precision: {disc['ulp_bits']} bits")

    console.print(f"\n[bold]Parameters:[/bold]")
    params = json.loads(disc["params_json"])
    for key, value in params.items():
        console.print(f"  {key}: {value}")

    console.print(f"\n[bold]Evidence:[/bold]")
    console.print(f"  Acceleration: {disc['accel_used']}")
    console.print(f"  Certification: {disc['cert_strategy']}")

    # Show recognition attempts
    recognition_attempts = database.get_recognition_attempts(discovery_id)
    if recognition_attempts:
        console.print(f"\n[bold]Recognition Attempts:[/bold]")
        for attempt in recognition_attempts[:3]:  # Show top 3
            success_marker = "[green]✓[/green]" if attempt["success"] else "[red]✗[/red]"
            console.print(f"  {success_marker} Basis: {attempt['basis_name']}")
            if attempt["success"] and attempt["relation_tex"]:
                console.print(f"    Relation: {attempt['relation_tex']}")
                console.print(f"    Height: {attempt['height']}")


@cli.command()
@click.option("--db", type=click.Path(exists=True, path_type=Path), default="ganita_data/math_discovery.db")
@click.option("--run-id", type=int, help="Specific run ID")
def stats(db: Path, run_id: int | None) -> None:
    """Show statistics for a run."""

    if not db.exists():
        console.print(f"[red]Database not found: {db}[/red]")
        sys.exit(1)

    database = Database(db)

    if not run_id:
        # Get latest run
        with database.connect() as conn:
            cur = conn.execute("SELECT MAX(run_id) as latest FROM runs")
            row = cur.fetchone()
            run_id = row["latest"] if row else None

    if not run_id:
        console.print("[yellow]No runs found in database[/yellow]")
        return

    stats_data = database.get_run_stats(run_id)

    console.print(Panel.fit(f"[bold cyan]Run Statistics - Run #{run_id}[/bold cyan]", border_style="cyan"))

    table = Table(show_header=False, border_style="cyan")
    table.add_column("Metric", style="bold")
    table.add_column("Value", justify="right")

    table.add_row("Total Discoveries", str(stats_data.get("total_discoveries", 0)))
    table.add_row("Certified", str(stats_data.get("certified", 0)))
    table.add_row("Recognized", str(stats_data.get("recognized", 0)))
    table.add_row("Cross-Validated", str(stats_data.get("cross_validated", 0)))
    table.add_row("Average Score", f"{stats_data.get('avg_score', 0):.4f}")
    table.add_row("Max Score", f"{stats_data.get('max_score', 0):.4f}")

    console.print(table)


def _show_run_summary(db: Path, run_id: int) -> None:
    """Show summary after a run completes."""
    database = Database(db)
    stats = database.get_run_stats(run_id)

    console.print("\n[bold]Summary:[/bold]")
    console.print(f"  Total discoveries: [cyan]{stats.get('total_discoveries', 0)}[/cyan]")
    console.print(f"  Certified: [green]{stats.get('certified', 0)}[/green]")
    console.print(f"  Recognized: [yellow]{stats.get('recognized', 0)}[/yellow]")
    console.print(f"  Cross-validated: [magenta]{stats.get('cross_validated', 0)}[/magenta]")
    console.print(f"  Average score: [blue]{stats.get('avg_score', 0):.4f}[/blue]")

    # Show top discoveries
    console.print("\n[bold]Top 5 Discoveries:[/bold]")
    top_discoveries = database.query_top(limit=5, run_id=run_id)

    for i, disc in enumerate(top_discoveries, 1):
        value_preview = disc["value_repr"][:50] + "..." if len(disc["value_repr"]) > 50 else disc["value_repr"]
        stage_style = _get_stage_style(disc["stage"])
        console.print(
            f"  {i}. {value_preview} "
            f"[{stage_style}]{disc['stage']}[/{stage_style}] "
            f"(score: {disc['rank_score']:.4f})"
        )


def _get_stage_style(stage: str) -> str:
    """Get color style for a stage."""
    styles = {
        "observed": "dim",
        "certified": "green",
        "recognized": "yellow",
        "cross_validated": "magenta",
        "novel_candidate": "cyan bold",
    }
    return styles.get(stage, "white")


@cli.command()
@click.option(
    "--config",
    "-c",
    type=click.Path(exists=True, path_type=Path),
    default="config.yaml",
    help="Configuration file path",
)
@click.option(
    "--db",
    type=click.Path(path_type=Path),
    default="ganita_data/math_discovery.db",
    help="Database path",
)
@click.option(
    "--artifacts",
    type=click.Path(path_type=Path),
    default="ganita_data/artifacts",
    help="Artifacts directory",
)
@click.option(
    "--checkpoint",
    type=click.Path(path_type=Path),
    default="ganita_data/checkpoint.json",
    help="Checkpoint file path",
)
@click.option(
    "--max-iterations",
    "-n",
    type=int,
    default=None,
    help="Maximum iterations (default: unlimited)",
)
@click.option(
    "--reset",
    is_flag=True,
    help="Reset checkpoint and start fresh",
)
def auto(
    config: Path, db: Path, artifacts: Path, checkpoint: Path, max_iterations: int | None, reset: bool
) -> None:
    """
    Run continuous auto-discovery mode.

    Explores parameter space indefinitely, resuming from checkpoint.
    Press Ctrl+C to stop gracefully.
    """
    console.print(
        Panel.fit(
            "[bold magenta]Ganit-A Auto-Discovery Mode[/bold magenta]\n"
            "[dim]Continuous exploration with checkpointing[/dim]",
            border_style="magenta",
        )
    )

    # Load configuration
    if config.exists():
        with open(config) as f:
            config_dict = yaml.safe_load(f)
        run_config = RunConfig.model_validate(config_dict)
    else:
        console.print(f"[yellow]Config file not found, using defaults[/yellow]")
        run_config = _create_default_config()

    # Handle reset
    if reset:
        if checkpoint.exists():
            checkpoint.unlink()
        console.print("[yellow]Checkpoint reset[/yellow]")

    # Show configuration
    console.print(f"\n[bold]Configuration:[/bold]")
    console.print(f"  Database: {db}")
    console.print(f"  Checkpoint: {checkpoint}")
    console.print(f"  Max iterations: {max_iterations or 'Unlimited'}")

    # Check if resuming
    if checkpoint.exists() and not reset:
        ckpt = ExplorationCheckpoint(checkpoint)
        progress = ckpt.get_progress()
        console.print(f"\n[bold cyan]Resuming from checkpoint:[/bold cyan]")
        console.print(f"  Total discoveries: {progress['total_discoveries']}")
        console.print(f"  Total iterations: {progress['total_iterations']}")
        console.print(f"  Status: {progress['status']}")

    console.print(
        "\n[bold green]Starting auto-discovery...[/bold green] (Press Ctrl+C to stop gracefully)"
    )

    # Create auto-discovery instance
    auto_discovery = AutoDiscovery(run_config, db, artifacts, checkpoint)

    # Start
    try:
        auto_discovery.start(continuous=True, max_iterations=max_iterations)
    except KeyboardInterrupt:
        console.print("\n[yellow]Interrupted by user[/yellow]")
    finally:
        console.print("\n[bold]Auto-discovery stopped.[/bold]")
        console.print(f"Checkpoint saved to: {checkpoint}")
        console.print("Run the same command to resume.")


@cli.command()
@click.option(
    "--checkpoint",
    type=click.Path(exists=True, path_type=Path),
    default="ganita_data/checkpoint.json",
    help="Checkpoint file path",
)
def checkpoint_status(checkpoint: Path) -> None:
    """Show checkpoint status and progress."""
    if not checkpoint.exists():
        console.print(f"[yellow]No checkpoint found at {checkpoint}[/yellow]")
        return

    ckpt = ExplorationCheckpoint(checkpoint)
    progress = ckpt.get_progress()

    console.print(Panel.fit("[bold cyan]Checkpoint Status[/bold cyan]", border_style="cyan"))

    # Overall stats
    table = Table(show_header=False, border_style="cyan")
    table.add_column("Metric", style="bold")
    table.add_column("Value")

    table.add_row("Status", progress["status"])
    table.add_row("Total Discoveries", str(progress["total_discoveries"]))
    table.add_row("Total Iterations", str(progress["total_iterations"]))
    table.add_row("Last Updated", progress["last_updated"])

    console.print(table)

    # Module progress
    if progress["modules"]:
        console.print("\n[bold]Module Progress:[/bold]")
        mod_table = Table(border_style="cyan")
        mod_table.add_column("Module", style="cyan")
        mod_table.add_column("Discoveries", justify="right")
        mod_table.add_column("Iterations", justify="right")
        mod_table.add_column("Explored", justify="right")
        mod_table.add_column("Status", style="green")

        for name, mod_progress in progress["modules"].items():
            mod_table.add_row(
                name,
                str(mod_progress["discoveries"]),
                str(mod_progress["iterations"]),
                str(mod_progress["explored_count"]),
                mod_progress["status"],
            )

        console.print(mod_table)

    # Export summary
    summary = ckpt.export_summary()
    console.print("\n[dim]Full summary exported to checkpoint.summary.txt[/dim]")


@cli.command()
@click.option(
    "--checkpoint",
    type=click.Path(exists=True, path_type=Path),
    default="ganita_data/checkpoint.json",
    help="Checkpoint file path",
)
@click.confirmation_option(prompt="Are you sure you want to reset the checkpoint?")
def checkpoint_reset(checkpoint: Path) -> None:
    """Reset checkpoint (start fresh)."""
    if checkpoint.exists():
        checkpoint.unlink()
        console.print(f"[green]✓[/green] Checkpoint reset: {checkpoint}")
    else:
        console.print(f"[yellow]No checkpoint found at {checkpoint}[/yellow]")


def _create_default_config() -> RunConfig:
    """Create default configuration."""
    return RunConfig(
        target_precision_digits=600,
        wallclock_limit_minutes=30.0,
        random_seed=42,
        max_workers=4,
        modules=[
            ModuleConfig(name="series_products", enabled=True, budget_minutes=10.0),
            ModuleConfig(name="continued_fractions", enabled=True, budget_minutes=10.0),
            ModuleConfig(name="fixed_points", enabled=True, budget_minutes=10.0),
        ],
    )


def main() -> None:
    """Entry point for CLI."""
    cli(obj={})


if __name__ == "__main__":
    main()
