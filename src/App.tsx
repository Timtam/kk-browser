import { invoke } from "@tauri-apps/api/core"
import { useEffect } from "react"
import { useNavigate } from "react-router"
import { Outlet } from "react-router-dom"
import "./App.css"

function App() {
    const navigate = useNavigate()

    useEffect(() => {
        ;(async () => {
            if (!(await invoke("db_found"))) navigate("/db-not-found")
        })()
    }, [navigate])

    return (
        <>
            <header>
                <h1>
                    KK Browser - the unofficial browser for Komplete Kontrol
                </h1>
            </header>
            <main className="container">
                <Outlet />
            </main>
        </>
    )
}

export default App
