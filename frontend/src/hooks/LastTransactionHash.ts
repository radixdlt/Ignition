import { useState } from "react"

export const useLastTransactionHash = () => {
    return useState<string | null>(null);
}