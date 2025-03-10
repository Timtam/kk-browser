import { invoke } from "@tauri-apps/api/core"
import natsort from "natsort"
import { useEffect, useMemo, useState } from "react"
import Accordion from "react-bootstrap/Accordion"
import Form from "react-bootstrap/Form"
import slugify from "slugify"
import { joinString } from "./utils"

const PAGE_SIZE = 50

interface Preset {
    name: string
    comment: string
    vendor: string
    product: String
    id: number
}

function Home() {
    const [loading, setLoading] = useState(true)
    const [vendors, setVendors] = useState<string[]>([])
    const [selectedVendors, setSelectedVendors] = useState<string[]>([])
    const [presets, setPresets] = useState<Preset[]>([])
    const [selectedPreset, setSelectedPreset] = useState(0)
    const [offset, setOffset] = useState(0)
    const sorter = useMemo(() => natsort(), [])

    useEffect(() => {
        ;(async () => {
            setVendors(await invoke("get_vendors"))
            const p = (await invoke("get_presets", {
                vendors: [],
                offset: 0,
                limit: PAGE_SIZE,
            })) as Preset[]
            setPresets(p)
            setSelectedPreset(p[0].id)
            setLoading(false)
        })()
    }, [setLoading, setPresets, setVendors])

    useEffect(() => {
        ;(async () => {
            const p = (await invoke("get_presets", {
                vendors: selectedVendors,
                offset: 0,
                limit: PAGE_SIZE,
            })) as Preset[]
            setOffset(0)
            setSelectedPreset(p[0].id)
            setPresets(p)
        })()
    }, [selectedVendors, setOffset, setPresets, setSelectedPreset])

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
            <select
                aria-label="Presets"
                onChange={async (e) => {
                    setSelectedPreset(parseInt(e.currentTarget.value, 10))
                    if (
                        presets.findIndex(
                            (p) => p.id === parseInt(e.currentTarget.value, 10),
                        ) >
                        offset - 10
                    ) {
                        const p = (await invoke("get_presets", {
                            vendors: selectedVendors,
                            offset: offset,
                            limit: PAGE_SIZE,
                        })) as Preset[]
                        setOffset(offset + p.length)
                        setPresets((old_presets) => old_presets.concat(p))
                    }
                }}
            >
                {presets.map((p) => {
                    return (
                        <option
                            key={p.id}
                            selected={p.id === selectedPreset}
                            id={p.id.toString()}
                            value={p.id}
                        >
                            {`${p.name}, ${p.comment}, ${p.product}, ${p.vendor}`}
                        </option>
                    )
                })}
            </select>
        </>
    )
}

export default Home
