#!/usr/bin/env python
"""
Ganit-A Demo Script

Demonstrates the mathematical discovery capabilities with simple examples.
"""

import logging
from pathlib import Path

import mpmath as mp
from rich.console import Console
from rich.panel import Panel
from rich.progress import Progress, SpinnerColumn, TextColumn
from rich.table import Table

from ganita.models import ModuleConfig, RunConfig
from ganita.orchestrator import run_exploration
from ganita.services.evaluator import Evaluator
from ganita.services.recognizer import Recognizer
from ganita.storage.db import Database

console = Console()
logging.basicConfig(level=logging.WARNING)


def demo_evaluator():
    """Demonstrate the high-precision evaluator."""
    console.print(Panel.fit("[bold cyan]Demo 1: High-Precision Evaluator[/bold cyan]", border_style="cyan"))

    evaluator = Evaluator({
        "precision_ramp": [128, 256],
        "stop_radius": "1e-100",
        "max_terms": 10000,
    })

    # Example 1: Basel problem (π²/6)
    console.print("\n[bold]Example 1: Basel Problem[/bold] Σ(1/n²)")

    def basel_term(n):
        return mp.mpf(1) / mp.mpf(n**2)

    result = evaluator.series_limit(basel_term, {"name": "basel"}, alternating=False)

    console.print(f"Computed value: {result.final.center[:50]}...")
    console.print(f"Expected (π²/6): {str(mp.pi**2 / 6)[:50]}...")
    console.print(f"Convergence: {'✓' if result.converged else '✗'}")
    console.print(f"Terms computed: {result.meta.get('terms_computed', 0)}")

    # Example 2: Alternating series
    console.print("\n[bold]Example 2: Alternating Series[/bold] Σ((-1)^n / n)")

    def alt_harmonic(n):
        return ((-1)**n) / mp.mpf(n)

    result = evaluator.series_limit(alt_harmonic, {"name": "alt_harmonic"}, alternating=True)

    console.print(f"Computed value: {result.final.center[:50]}...")
    console.print(f"Expected (ln 2): {str(mp.log(2))[:50]}...")
    console.print(f"Convergence: {'✓' if result.converged else '✗'}")


def demo_continued_fraction():
    """Demonstrate continued fraction evaluation."""
    console.print("\n" + "="*70)
    console.print(Panel.fit("[bold cyan]Demo 2: Continued Fractions[/bold cyan]", border_style="cyan"))

    evaluator = Evaluator({
        "precision_ramp": [128, 256],
        "stop_radius": "1e-100",
        "max_terms": 10000,
    })

    # Golden ratio: [1; 1, 1, 1, ...]
    console.print("\n[bold]Golden Ratio[/bold] φ = [1; 1, 1, 1, ...]")

    result = evaluator.continued_fraction(
        a_func=lambda n: mp.mpf(1),
        b_func=lambda n: mp.mpf(1),
        params={"name": "golden_ratio"}
    )

    phi = (1 + mp.sqrt(5)) / 2

    console.print(f"Computed value: {result.final.center[:50]}...")
    console.print(f"Expected φ: {str(phi)[:50]}...")
    console.print(f"Difference: {abs(mp.mpf(result.final.center) - phi)}")


def demo_fixed_point():
    """Demonstrate fixed point discovery."""
    console.print("\n" + "="*70)
    console.print(Panel.fit("[bold cyan]Demo 3: Fixed Points[/bold cyan]", border_style="cyan"))

    evaluator = Evaluator({
        "precision_ramp": [128, 256],
        "stop_radius": "1e-100",
        "max_terms": 10000,
    })

    # Dottie number: x = cos(x)
    console.print("\n[bold]Dottie Number[/bold] x = cos(x)")

    result = evaluator.fixed_point(
        f=lambda x: mp.cos(x),
        initial=mp.mpf(0.5),
        params={"name": "dottie"}
    )

    console.print(f"Fixed point: {result.final.center[:50]}...")
    console.print(f"Verification: cos({result.final.center[:20]}...) = {str(mp.cos(mp.mpf(result.final.center)))[:20]}...")
    console.print(f"Iterations: {result.meta.get('iterations', 0)}")


def demo_recognizer():
    """Demonstrate PSLQ constant recognition."""
    console.print("\n" + "="*70)
    console.print(Panel.fit("[bold cyan]Demo 4: Constant Recognition[/bold cyan]", border_style="cyan"))

    recognizer = Recognizer({
        "bases": [
            {
                "name": "small_v1",
                "symbols": ["1", "PI", "LOG2", "ZETA3", "CATALAN", "EULER_GAMMA"],
                "max_height": 2000,
            }
        ],
        "pslq_precision_digits": [200, 400],
    })

    # Test known values
    test_values = [
        (str(mp.pi**2 / 6), "π²/6 (Basel)"),
        (str(mp.log(2)), "ln(2)"),
        (str(mp.pi / 4), "π/4"),
    ]

    console.print("\n[bold]Testing Recognition on Known Constants:[/bold]\n")

    for value, description in test_values:
        console.print(f"[cyan]Testing:[/cyan] {description}")
        console.print(f"  Value: {value[:40]}...")

        result = recognizer.recognize(value)

        if result and result.success:
            console.print(f"  [green]✓ Recognized![/green]")
            console.print(f"  Relation: {result.relation_tex}")
            console.print(f"  Height: {result.height}")
            console.print(f"  Residual: 10^{result.residual_log10:.2f}")
        else:
            console.print(f"  [yellow]Not recognized[/yellow]")

        console.print()


def demo_full_run():
    """Demonstrate a full discovery run."""
    console.print("\n" + "="*70)
    console.print(Panel.fit("[bold cyan]Demo 5: Full Discovery Run[/bold cyan]", border_style="cyan"))

    # Create compact configuration
    config = RunConfig(
        target_precision_digits=200,  # Lower for demo speed
        wallclock_limit_minutes=5.0,  # Short run
        max_workers=2,
        modules=[
            ModuleConfig(
                name="series_products",
                enabled=True,
                budget_minutes=2.0,
                parameter_grid={
                    "template": ["dirichlet"],
                    "p": {"type": "range", "start": 2, "end": 4, "step": 1},
                    "a": [1],
                    "b": [0],
                    "sign": [1],
                }
            ),
            ModuleConfig(
                name="continued_fractions",
                enabled=True,
                budget_minutes=2.0,
                parameter_grid={
                    "template": ["simple"],
                    "a0": [0, 1],
                    "a_coeff": [1, 2],
                    "b_coeff": [1],
                }
            ),
        ],
    )

    # Setup paths
    db_path = Path("demo_data/demo.db")
    artifacts_path = Path("demo_data/artifacts")

    # Clean previous run
    if db_path.exists():
        db_path.unlink()
    if artifacts_path.exists():
        import shutil
        shutil.rmtree(artifacts_path)

    console.print("\n[bold]Starting discovery run...[/bold]")
    console.print(f"Configuration: {len(config.modules)} modules, {config.wallclock_limit_minutes} min limit")

    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        console=console,
    ) as progress:
        task = progress.add_task("[cyan]Discovering constants...", total=None)

        run_id = run_exploration(config, db_path, artifacts_path)

        progress.update(task, completed=True)

    console.print(f"\n[green]✓ Run completed![/green] Run ID: {run_id}")

    # Display results
    db = Database(db_path)
    stats = db.get_run_stats(run_id)

    console.print("\n[bold]Run Statistics:[/bold]")
    stats_table = Table(show_header=False, border_style="cyan")
    stats_table.add_column("Metric", style="bold")
    stats_table.add_column("Value")

    stats_table.add_row("Total Discoveries", str(stats.get("total_discoveries", 0)))
    stats_table.add_row("Certified", str(stats.get("certified", 0)))
    stats_table.add_row("Recognized", str(stats.get("recognized", 0)))
    stats_table.add_row("Cross-Validated", str(stats.get("cross_validated", 0)))
    stats_table.add_row("Average Score", f"{stats.get('avg_score', 0):.4f}")

    console.print(stats_table)

    # Show top discoveries
    top = db.query_top(limit=5, run_id=run_id)

    if top:
        console.print("\n[bold]Top 5 Discoveries:[/bold]")
        disc_table = Table(border_style="cyan")
        disc_table.add_column("ID", style="cyan")
        disc_table.add_column("Value", max_width=40)
        disc_table.add_column("Stage", style="green")
        disc_table.add_column("Score")

        for disc in top:
            value_str = disc["value_repr"][:40] + "..." if len(disc["value_repr"]) > 40 else disc["value_repr"]
            disc_table.add_row(
                str(disc["discovery_id"]),
                value_str,
                disc["stage"],
                f"{disc['rank_score']:.4f}"
            )

        console.print(disc_table)


def main():
    """Run all demos."""
    console.print(Panel.fit(
        "[bold magenta]Ganit-A Mathematical Discovery Engine[/bold magenta]\n"
        "[dim]Interactive Demo[/dim]",
        border_style="magenta"
    ))

    try:
        demo_evaluator()
        demo_continued_fraction()
        demo_fixed_point()
        demo_recognizer()
        demo_full_run()

        console.print("\n" + "="*70)
        console.print(Panel.fit(
            "[bold green]✓ All demos completed successfully![/bold green]",
            border_style="green"
        ))

    except Exception as e:
        console.print(f"\n[bold red]Error:[/bold red] {e}")
        console.print_exception()


if __name__ == "__main__":
    main()
