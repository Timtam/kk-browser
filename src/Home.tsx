import { invoke } from "@tauri-apps/api/core"
import { writeText } from "@tauri-apps/plugin-clipboard-manager"
import natsort from "natsort"
import { useEffect, useMemo, useState } from "react"
import Button from "react-bootstrap/Button"
import Form from "react-bootstrap/Form"
import Modal from "react-bootstrap/Modal"
import { AsyncPaginate as Select } from "react-select-async-paginate"
import slugify from "slugify"
import { joinString } from "./utils"

const PAGE_SIZE = 500

interface Preset {
    name: string
    comment: string
    vendor: string
    product_name: String
    id: number
    file_name: string
    categories: number[]
    modes: number[]
    bank: number
}

interface Product {
    name: string
    vendor: string
    id: number
}

interface Category {
    id: number
    name: string
    subcategory: string
    subsubcategory: string
}

interface Mode {
    id: number
    name: string
}

interface Bank {
    id: number
    entry1: string
    entry2: string
    entry3: string
}

interface PaginatedResult<T> {
    results: T[]
    total: number
    start: number
    end: number
}

interface PresetOption extends Preset {
    label: string
}

function Home() {
    const [loading, setLoading] = useState(true)
    const [vendors, setVendors] = useState<string[]>([])
    const [selectedVendors, setSelectedVendors] = useState<string[]>([])
    const [temporarilySelectedVendors, setTemporarilySelectedVendors] =
        useState<string[]>([])
    const [products, setProducts] = useState<Map<number, Product>>(new Map())
    const [selectedProducts, setSelectedProducts] = useState<number[]>([])
    const [temporarilySelectedProducts, setTemporarilySelectedProducts] =
        useState<number[]>([])
    const sorter = useMemo(natsort, [])
    const [selectedPreset, setSelectedPreset] = useState<
        PresetOption | undefined
    >(undefined)
    const [categories, setCategories] = useState<Map<number, Category>>(
        new Map(),
    )
    const [selectedCategories, setSelectedCategories] = useState<number[]>([])
    const [temporarilySelectedCategories, setTemporarilySelectedCategories] =
        useState<number[]>([])
    const [banks, setBanks] = useState<Map<number, Bank>>(new Map())
    const [selectedBanks, setSelectedBanks] = useState<number[]>([])
    const [temporarilySelectedBanks, setTemporarilySelectedBanks] = useState<
        number[]
    >([])
    const [showProducts, setShowProducts] = useState(false)
    const [showVendors, setShowVendors] = useState(false)
    const [showCategories, setShowCategories] = useState(false)
    const [showModes, setShowModes] = useState(false)
    const [showBanks, setShowBanks] = useState(false)
    const [query, setQuery] = useState("")
    const [modes, setModes] = useState<Map<number, Mode>>(new Map())
    const [selectedModes, setSelectedModes] = useState<number[]>([])
    const [temporarilySelectedModes, setTemporarilySelectedModes] = useState<
        number[]
    >([])
    const [bankFilter, setBankFilter] = useState("")

    useEffect(() => {
        ;(async () => {
            if (loading) {
                let server_loading = true

                while (server_loading) {
                    server_loading = await invoke("is_loading")
                    if (server_loading)
                        await new Promise((r) => setTimeout(r, 100))
                }
                setLoading(false)
            }
        })()
    }, [loading, setLoading])

    useEffect(() => {
        ;(async () => {
            if (!loading) {
                setVendors(
                    await invoke("get_vendors", {
                        products: selectedProducts,
                        categories: selectedCategories,
                        modes: selectedModes,
                        banks: selectedBanks,
                    }),
                )
                setProducts(
                    new Map(
                        (
                            (await invoke("get_products", {
                                vendors: selectedVendors,
                                categories: selectedCategories,
                                modes: selectedModes,
                                banks: selectedBanks,
                            })) as Product[]
                        ).map((p) => [p.id, p]),
                    ),
                )
                setCategories(
                    new Map(
                        (
                            (await invoke("get_categories", {
                                vendors: selectedVendors,
                                products: selectedProducts,
                                modes: selectedModes,
                                banks: selectedBanks,
                            })) as Category[]
                        ).map((c) => [c.id, c]),
                    ),
                )
                setModes(
                    new Map(
                        (
                            (await invoke("get_modes", {
                                vendors: selectedVendors,
                                products: selectedProducts,
                                categories: selectedCategories,
                                banks: selectedBanks,
                            })) as Mode[]
                        ).map((m) => [m.id, m]),
                    ),
                )
                setBanks(
                    new Map(
                        (
                            (await invoke("get_banks", {
                                vendors: selectedVendors,
                                products: selectedProducts,
                                categories: selectedCategories,
                                modes: selectedModes,
                            })) as Bank[]
                        ).map((b) => [b.id, b]),
                    ),
                )
            }
        })()
    }, [
        loading,
        selectedBanks,
        selectedCategories,
        selectedModes,
        selectedProducts,
        selectedVendors,
        setBanks,
        setCategories,
        setModes,
        setProducts,
        setVendors,
    ])

    return loading ? (
        <p>Loading Komplete Kontrol data, please wait...</p>
    ) : (
        <>
            <section aria-label="Filter">
                <h2>Filter presets</h2>
                <Button
                    aria-expanded={false}
                    onClick={() => setShowVendors(true)}
                >
                    Vendors:{" "}
                    {selectedVendors.length === 0
                        ? "All"
                        : joinString(
                              selectedVendors.sort(sorter),
                              ", ",
                              " and ",
                          )}
                </Button>
                <Modal
                    show={showVendors}
                    onHide={() => {
                        setShowVendors(false)
                        setSelectedVendors(temporarilySelectedVendors)
                    }}
                >
                    <Modal.Header closeButton closeLabel="Save">
                        <Modal.Title>Vendors</Modal.Title>
                    </Modal.Header>
                    <Modal.Body>
                        <Button
                            onClick={() => setTemporarilySelectedVendors([])}
                        >
                            Deselect all
                        </Button>
                        <div role="list" aria-label="Vendors">
                            {vendors!
                                .filter(
                                    (v) =>
                                        selectedProducts.length === 0 ||
                                        selectedProducts
                                            .map((p) => products.get(p)!)
                                            .find((p) => p.vendor === v) !==
                                            undefined,
                                )
                                .sort(sorter)
                                .map((v, i) => (
                                    <div role="listitem">
                                        <Form.Check
                                            type="checkbox"
                                            id={`${slugify(v)}-${i}`}
                                            key={`${slugify(v)}-${i}`}
                                            label={v}
                                            checked={temporarilySelectedVendors.includes(
                                                v,
                                            )}
                                            onChange={() =>
                                                temporarilySelectedVendors.includes(
                                                    v,
                                                )
                                                    ? setTemporarilySelectedVendors(
                                                          temporarilySelectedVendors.filter(
                                                              (v2) => v !== v2,
                                                          ),
                                                      )
                                                    : setTemporarilySelectedVendors(
                                                          [
                                                              ...temporarilySelectedVendors,
                                                              v,
                                                          ],
                                                      )
                                            }
                                        />
                                    </div>
                                ))}
                        </div>
                    </Modal.Body>
                </Modal>
                <Button
                    aria-expanded={false}
                    onClick={() => setShowProducts(true)}
                >
                    Products:{" "}
                    {selectedProducts.length === 0
                        ? "All"
                        : joinString(
                              selectedProducts
                                  .map(
                                      (p) =>
                                          products.get(p)!.name ||
                                          "(unnamed product)",
                                  )
                                  .sort(sorter),
                              ", ",
                              " and ",
                          )}
                </Button>
                <Modal
                    show={showProducts}
                    onHide={() => {
                        setShowProducts(false)
                        setSelectedProducts(temporarilySelectedProducts)
                    }}
                >
                    <Modal.Header closeButton closeLabel="Save">
                        <Modal.Title>Products</Modal.Title>
                    </Modal.Header>
                    <Modal.Body>
                        <Button
                            onClick={() => setTemporarilySelectedProducts([])}
                        >
                            Deselect all
                        </Button>
                        <div role="list" aria-label="Products">
                            {[...products!.values()].map((p, i) => (
                                <div role="listitem">
                                    <Form.Check
                                        type="checkbox"
                                        id={`${slugify(p.name)}-${i}`}
                                        label={p.name || "(unnamed product)"}
                                        checked={temporarilySelectedProducts.includes(
                                            p.id,
                                        )}
                                        onChange={() =>
                                            temporarilySelectedProducts.includes(
                                                p.id,
                                            )
                                                ? setTemporarilySelectedProducts(
                                                      temporarilySelectedProducts.filter(
                                                          (p2) => p.id !== p2,
                                                      ),
                                                  )
                                                : setTemporarilySelectedProducts(
                                                      [
                                                          ...temporarilySelectedProducts,
                                                          p.id,
                                                      ],
                                                  )
                                        }
                                    />
                                </div>
                            ))}
                        </div>
                    </Modal.Body>
                </Modal>
                <Button
                    aria-expanded={false}
                    onClick={() => setShowBanks(true)}
                >
                    Banks:{" "}
                    {selectedBanks.length === 0
                        ? "All"
                        : joinString(
                              selectedBanks
                                  .map((b) =>
                                      joinString(
                                          [
                                              banks.get(b)!.entry1,
                                              banks.get(b)!.entry2,
                                              banks.get(b)!.entry3,
                                          ].filter((b) => b !== ""),
                                          " / ",
                                      ),
                                  )
                                  .sort(sorter),
                              ", ",
                              " and ",
                          )}
                </Button>
                <Modal
                    show={showBanks}
                    onHide={() => {
                        setShowBanks(false)
                        setSelectedBanks(temporarilySelectedBanks)
                    }}
                >
                    <Modal.Header closeButton closeLabel="Save">
                        <Modal.Title>Banks</Modal.Title>
                    </Modal.Header>
                    <Modal.Body>
                        <p>
                            <Button
                                onClick={() => {
                                    setBankFilter("")
                                    setTemporarilySelectedBanks([])
                                }}
                            >
                                Deselect all
                            </Button>
                        </p>
                        <Form.Group controlId="BankFilter">
                            <Form.Label>Filter</Form.Label>
                            <Form.Control
                                type="input"
                                placeholder="Filter banks..."
                                onChange={(e) => setBankFilter(e.target.value)}
                            />
                        </Form.Group>
                        <div role="list" aria-label="Banks">
                            {[...banks!.values()]
                                .filter(
                                    (b) =>
                                        bankFilter.trim() === "" ||
                                        b.entry1
                                            .toLowerCase()
                                            .includes(
                                                bankFilter.toLowerCase(),
                                            ) ||
                                        b.entry2
                                            .toLowerCase()
                                            .includes(
                                                bankFilter.toLowerCase(),
                                            ) ||
                                        b.entry3
                                            .toLowerCase()
                                            .includes(bankFilter.toLowerCase()),
                                )
                                .map((b, i) => (
                                    <div role="listitem">
                                        <Form.Check
                                            type="checkbox"
                                            id={`${slugify(b.entry1)}-${i}`}
                                            label={joinString(
                                                [
                                                    b.entry1,
                                                    b.entry2,
                                                    b.entry3,
                                                ].filter((b) => b !== ""),
                                                " / ",
                                            )}
                                            checked={temporarilySelectedBanks.includes(
                                                b.id,
                                            )}
                                            onChange={() =>
                                                temporarilySelectedBanks.includes(
                                                    b.id,
                                                )
                                                    ? setTemporarilySelectedBanks(
                                                          temporarilySelectedBanks.filter(
                                                              (b2) =>
                                                                  b.id !== b2,
                                                          ),
                                                      )
                                                    : setTemporarilySelectedBanks(
                                                          [
                                                              ...temporarilySelectedBanks,
                                                              b.id,
                                                          ],
                                                      )
                                            }
                                        />
                                    </div>
                                ))}
                        </div>
                    </Modal.Body>
                </Modal>
                <Button
                    aria-expanded={false}
                    onClick={() => setShowCategories(true)}
                >
                    Types:{" "}
                    {selectedCategories.length === 0
                        ? "All"
                        : joinString(
                              selectedCategories
                                  .map((c) =>
                                      joinString(
                                          [
                                              categories.get(c)!.name,
                                              categories.get(c)!.subcategory,
                                              categories.get(c)!.subsubcategory,
                                          ].filter((c) => c !== ""),
                                          " / ",
                                      ),
                                  )
                                  .sort(sorter),
                              ", ",
                              " and ",
                          )}
                </Button>
                <Modal
                    show={showCategories}
                    onHide={() => {
                        setShowCategories(false)
                        setSelectedCategories(temporarilySelectedCategories)
                    }}
                >
                    <Modal.Header closeButton closeLabel="Save">
                        <Modal.Title>Types</Modal.Title>
                    </Modal.Header>
                    <Modal.Body>
                        <Button
                            onClick={() => setTemporarilySelectedCategories([])}
                        >
                            Deselect all
                        </Button>
                        <div role="list" aria-label="Types">
                            {[...categories!.values()].map((c, i) => (
                                <div role="listitem">
                                    <Form.Check
                                        type="checkbox"
                                        id={`${slugify(c.name)}-${i}`}
                                        label={joinString(
                                            [
                                                c.name,
                                                c.subcategory,
                                                c.subsubcategory,
                                            ].filter((c) => c !== ""),
                                            " / ",
                                        )}
                                        checked={temporarilySelectedCategories.includes(
                                            c.id,
                                        )}
                                        onChange={() =>
                                            temporarilySelectedCategories.includes(
                                                c.id,
                                            )
                                                ? setTemporarilySelectedCategories(
                                                      temporarilySelectedCategories.filter(
                                                          (c2) => c.id !== c2,
                                                      ),
                                                  )
                                                : setTemporarilySelectedCategories(
                                                      [
                                                          ...temporarilySelectedCategories,
                                                          c.id,
                                                      ],
                                                  )
                                        }
                                    />
                                </div>
                            ))}
                        </div>
                    </Modal.Body>
                </Modal>
                <Button
                    aria-expanded={false}
                    onClick={() => setShowModes(true)}
                >
                    Characteristics:{" "}
                    {selectedModes.length === 0
                        ? "All"
                        : joinString(
                              selectedModes
                                  .map((m) => modes.get(m)!.name)
                                  .sort(sorter),
                              ", ",
                              " and ",
                          )}
                </Button>
                <Modal
                    show={showModes}
                    onHide={() => {
                        setShowModes(false)
                        setSelectedModes(temporarilySelectedModes)
                    }}
                >
                    <Modal.Header closeButton closeLabel="Save">
                        <Modal.Title>Characteristics</Modal.Title>
                    </Modal.Header>
                    <Modal.Body>
                        <Button onClick={() => setTemporarilySelectedModes([])}>
                            Deselect all
                        </Button>
                        <div role="list" aria-label="Characteristics">
                            {[...modes!.values()].map((m, i) => (
                                <div role="listitem">
                                    <Form.Check
                                        type="checkbox"
                                        id={`${slugify(m.name)}-${i}`}
                                        label={m.name}
                                        checked={temporarilySelectedModes.includes(
                                            m.id,
                                        )}
                                        onChange={() =>
                                            temporarilySelectedModes.includes(
                                                m.id,
                                            )
                                                ? setTemporarilySelectedModes(
                                                      temporarilySelectedModes.filter(
                                                          (m2) => m.id !== m2,
                                                      ),
                                                  )
                                                : setTemporarilySelectedModes([
                                                      ...temporarilySelectedModes,
                                                      m.id,
                                                  ])
                                        }
                                    />
                                </div>
                            ))}
                        </div>
                    </Modal.Body>
                </Modal>
            </section>
            <section aria-label="Results">
                <h2>Results</h2>
                <Select
                    closeMenuOnSelect={false}
                    inputValue={query}
                    onInputChange={(value, action) => {
                        if (action.action === "input-change") {
                            setQuery(value)
                        }
                        if (action.action === "set-value") {
                            return query
                        }
                        if (action.action === "menu-close") {
                            return action.prevInputValue
                        }
                    }}
                    cacheUniqs={[
                        selectedBanks,
                        selectedCategories,
                        selectedModes,
                        selectedProducts,
                        selectedVendors,
                    ]}
                    value={selectedPreset}
                    isMulti={false}
                    isSearchable={true}
                    loadOptions={async (query: string, loadedOptions) => {
                        let res = (await invoke("get_presets", {
                            vendors: selectedVendors,
                            products: selectedProducts,
                            categories: selectedCategories,
                            modes: selectedModes,
                            banks: selectedBanks,
                            query: query,
                            offset: loadedOptions.length,
                            limit: PAGE_SIZE,
                        })) as PaginatedResult<Preset>

                        return {
                            options: res.results.map((p) => ({
                                ...p,
                                label: `${p.name}, ${p.comment || "no description"}, ${p.product_name}`,
                            })),
                            hasMore: res.total > res.end,
                        }
                    }}
                    aria-label="Presets"
                    onChange={(o) => {
                        ;(async () => {
                            setSelectedPreset(o!)
                            await invoke("play_preset", {
                                preset: o!.id,
                            })
                        })()
                    }}
                />
            </section>
            <section aria-label="Preset details">
                {selectedPreset === undefined ? (
                    <p>No preset selected</p>
                ) : (
                    <>
                        <h2>Preset details for {selectedPreset.name}</h2>
                        <p>Vendor: {selectedPreset.vendor}</p>
                        <p>Product: {selectedPreset.product_name}</p>
                        <p>
                            Bank:{" "}
                            {selectedPreset.bank === 0
                                ? "none"
                                : joinString(
                                      [
                                          banks.get(selectedPreset.bank)!
                                              .entry1,
                                          banks.get(selectedPreset.bank)!
                                              .entry2,
                                          banks.get(selectedPreset.bank)!
                                              .entry3,
                                      ].filter((s) => s !== ""),
                                      " / ",
                                  )}
                        </p>
                        <p>
                            Types:{" "}
                            {joinString(
                                selectedPreset.categories
                                    .map((c) => {
                                        let cat = categories.get(c)!

                                        return joinString(
                                            [
                                                cat.name,
                                                cat.subcategory,
                                                cat.subsubcategory,
                                            ].filter((s) => s !== ""),
                                            " / ",
                                        )
                                    })
                                    .sort(sorter),
                                ", ",
                                " and ",
                            ) || "none"}
                        </p>
                        <p>
                            Characteristics:{" "}
                            {joinString(
                                selectedPreset.modes
                                    .map((m) => modes.get(m)!.name)
                                    .sort(sorter),
                                ", ",
                                " and ",
                            ) || "none"}
                        </p>
                        <Button
                            onClick={async () =>
                                await writeText(selectedPreset!.file_name)
                            }
                        >
                            Copy file path to clipboard
                        </Button>
                    </>
                )}
            </section>
        </>
    )
}

export default Home
