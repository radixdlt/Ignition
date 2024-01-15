import { useState, useEffect } from 'react';
import { switchMap, map } from 'rxjs'
import { dAppToolkit } from '../RadixDappToolkit';

export const useConnectedAccounts = () => {
  const [state, setState] = useState<ConnectedAccount>({ status: 'pending' })

  useEffect(() => {
    const subscription = dAppToolkit.walletApi.walletData$
      .pipe(
        map((walletData) => walletData.accounts),
        switchMap((accounts) => {
          if (accounts?.[0]?.address !== undefined) {
            setState({ status: 'success', account: accounts?.[0]?.address })
          }

          return accounts
        })
      )
      .subscribe()

    return () => {
      subscription.unsubscribe()
    }
  }, [dAppToolkit])

  return state
}

export type ConnectedAccount =
  { status: 'success', account: string }
  | { status: 'pending' }
  | { status: 'error' }
