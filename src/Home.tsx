import { invoke } from "@tauri-apps/api/core"
import natsort from "natsort"
import { useEffect, useMemo, useState } from "react"
import Accordion from "react-bootstrap/Accordion"
import Form from "react-bootstrap/Form"
import slugify from "slugify"
import { joinString } from "./utils"

function Home() {
    const [loading, setLoading] = useState(true)
    const [vendors, setVendors] = useState<String[]>([])
    const [selectedVendors, setSelectedVendors] = useState<string[]>([])
    const sorter = useMemo(() => natsort(), [])

    useEffect(() => {
        ;(async () => {
            setVendors(await invoke("get_vendors"))
            setLoading(false)
        })()
    }, [setLoading, setVendors])

    return loading ? (
        <p>Loading...</p>
    ) : (
        <>
            <Accordion>
                <Accordion.Item eventKey="vendors">
                    <Accordion.Header as="p">
                        Vendors:{" "}
                        {selectedVendors.length === 0
                            ? "All"
                            : joinString(
                                  selectedVendors.sort(sorter),
                                  ", ",
                                  " and ",
                              )}
                    </Accordion.Header>
                    <Accordion.Body>
                        <div role="list" aria-label="Vendors">
                            {vendors!.sort(sorter).map((v, i) => (
                                <div role="listitem">
                                    <Form.Check
                                        type="checkbox"
                                        id={`${slugify(v)}-${i}`}
                                        label={v}
                                        checked={selectedVendors.includes(v)}
                                        onChange={() =>
                                            selectedVendors.includes(v)
                                                ? setSelectedVendors(
                                                      selectedVendors.filter(
                                                          (v2) => v !== v2,
                                                      ),
                                                  )
                                                : setSelectedVendors([
                                                      ...selectedVendors,
                                                      v,
                                                  ])
                                        }
                                    />
                                </div>
                            ))}
                        </div>
                    </Accordion.Body>
                </Accordion.Item>
            </Accordion>
        </>
    )
}

export default Home
