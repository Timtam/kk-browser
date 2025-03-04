import "bootstrap/dist/css/bootstrap.min.css"
import React from "react"
import ReactDOM from "react-dom/client"
import { RouterProvider, createBrowserRouter } from "react-router-dom"
import App from "./App"
import DbNotFound from "./DbNotFound"
import Home from "./Home"

const router = createBrowserRouter([
    {
        element: <App />,
        children: [
            {
                element: <DbNotFound />,
                path: "/db-not-found",
            },
            {
                element: <Home />,
                path: "/",
            },
        ],
    },
])

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
        <RouterProvider router={router} />
    </React.StrictMode>,
)
