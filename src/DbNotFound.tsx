import { invoke } from "@tauri-apps/api/core"
import { exit } from "@tauri-apps/plugin-process"
import { useEffect, useState } from "react"

function DbNotFound() {
    let [dbPath, setDbPath] = useState("")

    useEffect(() => {
        ;(async () => {
            setDbPath(await invoke("get_db_path"))
        })()
    }, [setDbPath])

    return (
        <>
            <h2>Database not found</h2>
            <p>
                The komplete.db3 could not be found on your system. Please exit
                the app and make sure to run Komplete Kontrol standalone first
                to create a database that we can use.
            </p>
            <p>
                This application is looking in the following path to find the
                database file: {dbPath}
            </p>
            <button onClick={async () => exit(0)}>Exit</button>
        </>
    )
}

export default DbNotFound
