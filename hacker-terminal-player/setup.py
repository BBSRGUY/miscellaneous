from setuptools import setup, find_packages

setup(
    name="hacker-terminal-player",
    version="1.0.0",
    description="A retro hacker-themed terminal video player",
    packages=find_packages(),
    install_requires=[
        "opencv-python>=4.8.0",
        "numpy>=1.24.0",
        "pygame>=2.5.0",
        "blessed>=1.20.0",
    ],
    entry_points={
        "console_scripts": [
            "vid-player=src.main:main",
        ],
    },
    python_requires=">=3.8",
)
