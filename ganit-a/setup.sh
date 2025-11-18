#!/bin/bash
# Ganit-A Quick Setup Script

set -e

echo "╔════════════════════════════════════════════════════════════╗"
echo "║       Ganit-A: Mathematical Discovery Engine Setup        ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Check Python version
echo "Checking Python version..."
python_version=$(python3 --version 2>&1 | awk '{print $2}')
required_version="3.11"

if ! python3 -c "import sys; exit(0 if sys.version_info >= (3, 11) else 1)"; then
    echo "❌ Python 3.11 or higher is required. You have Python $python_version"
    exit 1
fi

echo "✓ Python $python_version found"
echo ""

# Create virtual environment
echo "Creating virtual environment..."
if [ ! -d "venv" ]; then
    python3 -m venv venv
    echo "✓ Virtual environment created"
else
    echo "✓ Virtual environment already exists"
fi
echo ""

# Activate virtual environment
echo "Activating virtual environment..."
source venv/bin/activate
echo "✓ Virtual environment activated"
echo ""

# Upgrade pip
echo "Upgrading pip..."
pip install --upgrade pip > /dev/null 2>&1
echo "✓ pip upgraded"
echo ""

# Install package
echo "Installing Ganit-A..."
pip install -e . > /dev/null 2>&1
echo "✓ Ganit-A installed"
echo ""

# Create data directories
echo "Creating data directories..."
mkdir -p ganita_data/artifacts
mkdir -p demo_data/artifacts
mkdir -p reports
echo "✓ Data directories created"
echo ""

# Verify installation
echo "Verifying installation..."
if ganit-a --version > /dev/null 2>&1; then
    version=$(ganit-a --version 2>&1 | grep -oP '\d+\.\d+\.\d+')
    echo "✓ Ganit-A CLI v$version is working"
else
    echo "❌ CLI verification failed"
    exit 1
fi
echo ""

# Display next steps
echo "╔════════════════════════════════════════════════════════════╗"
echo "║                   Setup Complete! 🎉                       ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "Next steps:"
echo ""
echo "1. Activate the virtual environment:"
echo "   source venv/bin/activate"
echo ""
echo "2. Run the interactive demo:"
echo "   python examples/demo.py"
echo ""
echo "3. Start a discovery run:"
echo "   ganit-a run --config config.yaml"
echo ""
echo "4. Start the web server:"
echo "   uvicorn ganita.api:app --reload"
echo "   Then open web/index.html in your browser"
echo ""
echo "5. View help:"
echo "   ganit-a --help"
echo ""
echo "Happy discovering! 🔬✨"
echo ""
