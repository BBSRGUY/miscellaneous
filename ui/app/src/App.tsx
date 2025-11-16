import { useState } from 'react'
import './App.css'

function App() {
  const [greeting, setGreeting] = useState('')

  async function greet() {
    // When running in Tauri, use the invoke API
    // For now, just set a static greeting
    setGreeting('Hello from Forge!')
  }

  return (
    <div className="container">
      <h1>Forge</h1>
      <p>Local LLM Platform</p>

      <div className="card">
        <button onClick={greet}>
          Click to greet
        </button>
        {greeting && <p>{greeting}</p>}
      </div>

      <p className="read-the-docs">
        Welcome to Forge - your local LLM platform
      </p>
    </div>
  )
}

export default App
