import { describe, expect, it } from "vitest"
import type { AgentChatMessage } from "../types"
import { getStudentNextActionGroup } from "./student-next-actions-view-model"
import { textFromCodeString } from "./student-next-actions-view-model-test-utils"

describe("student next action learning domain", () => {
  it("keeps GA4 checkout analysis aligned with the conversion learning domain", () => {
    const previousUserContent = textFromCodeString(
      "26576 21830 21697 35814 24773 39029 35775 38382 37327 27491 24120 65292 20294 24456 22810 29992 25143 36827 20837 32467 36134 27969 31243 21518 27809 26377 36141 20080 12290 35831 21578 35785 25105 24212 35813 26816 26597 21738 20123 32 71 65 52 32 30005 21830 20107 20214 21644 36716 21270 29615 33410 12290"
    )
    const message: AgentChatMessage = {
      id: "assistant-ga4-checkout",
      role: "assistant",
      content: textFromCodeString(
        "20808 26816 26597 32 118 105 101 119 95 105 116 101 109 12289 97 100 100 95 116 111 95 99 97 114 116 12289 98 101 103 105 110 95 99 104 101 99 107 111 117 116 12289 97 100 100 95 115 104 105 112 112 105 110 103 95 105 110 102 111 12289 97 100 100 95 112 97 121 109 101 110 116 95 105 110 102 111 32 21644 32 112 117 114 99 104 97 115 101 65292 20877 27604 36739 21508 29615 33410 36716 21270 29575 21644 20107 20214 21442 25968 12290"
      ),
    }

    expect(getStudentNextActionGroup(message, previousUserContent)).toBe("data")
  })

  it("keeps personalized recommendation rights aligned with compliance", () => {
    const previousUserContent = textFromCodeString(
      "22312 20013 22269 30005 21830 24179 21488 20570 20010 24615 21270 25512 33616 26102 65292 20851 38381 20010 24615 21270 25512 33616 30340 20837 21475 24212 35813 24590 20040 35774 35745 65311 20010 20154 20449 24687 21644 31639 27861 25512 33616 21512 35268 19978 26377 21738 20123 35201 27714 65311"
    )
    const message: AgentChatMessage = {
      id: "assistant-personalized-recommendation",
      role: "assistant",
      content: textFromCodeString(
        "20851 38381 20010 24615 21270 25512 33616 30340 20837 21475 35201 26174 33879 26131 25214 65292 38544 31169 35774 32622 20013 25552 20379 20010 24615 21270 25512 33616 31649 29702 12290 20851 38381 21518 19981 33021 32487 32493 29992 20010 20154 20449 24687 20570 25512 33616 65292 35201 23562 37325 29992 25143 36873 25321 26435 65292 24182 20381 25454 30005 23376 21830 21153 27861 20445 25252 28040 36153 32773 26435 30410 12290"
      ),
    }

    expect(getStudentNextActionGroup(message, previousUserContent)).toBe(
      "compliance"
    )
  })

  it("keeps deposit refund turns in service when the answer also mentions data diagnostics", () => {
    const previousUserContent =
      "Explain reasonable deposit refund conditions in a consumer dispute."
    const message: AgentChatMessage = {
      id: "assistant-deposit-refund",
      role: "assistant",
      content:
        "Customer support should explain the deposit refund process and retain data diagnostics.",
    }

    expect(getStudentNextActionGroup(message, previousUserContent)).toBe(
      "service"
    )
  })

  it("keeps a title turn in product follow-up when the answer mentions service policy", () => {
    const previousUserContent = textFromCodeString(
      "29616 22312 35831 24110 25105 20248 21270 19968 27454 21654 21857 30340 21830 21697 26631 39064 65292 37325 28857 20889 28165 50 48 26465 35013 35268 26684 65292 24182 36991 20813 20419 38144 35789 22534 30732 12290"
    )
    const message: AgentChatMessage = {
      id: "assistant-coffee-title",
      role: "assistant",
      content: textFromCodeString(
        "26631 39064 24212 20889 28165 21654 21857 35268 26684 21644 21475 21619 65292 19981 35201 28151 20837 36864 36135 25919 31574 12289 21806 21518 35828 26126 25110 20419 38144 26085 26399 12290"
      ),
    }

    expect(getStudentNextActionGroup(message, previousUserContent)).toBe(
      "productFollowup"
    )
  })

  it("keeps inventory stocktake turns out of after-sales suggestions", () => {
    const previousUserContent = textFromCodeString(
      "21830 21697 24211 23384 30424 28857 21518 21457 29616 36134 23454 19981 19968 33268 65292 31532 19968 27493 24212 35813 24590 20040 26597 65311"
    )
    const message: AgentChatMessage = {
      id: "assistant-stocktake",
      role: "assistant",
      content: textFromCodeString(
        "20808 26680 23545 24211 23384 21644 20986 20837 24211 35760 24405 65292 20877 25490 26597 36864 36135 12289 25298 25910 21644 37319 36141 20837 24211 26159 21542 28431 35760 12290"
      ),
    }

    expect(getStudentNextActionGroup(message, previousUserContent)).toBe(
      "productNoMaterial"
    )
  })

  it("keeps an ordinary product recommendation in the product group", () => {
    const productRecommendation = textFromCodeString(
      "25512 33616 19968 27454 36866 21512 23398 29983 20154 32676 12289 20215 26684 22312 49 50 57 20803 24038 21491 30340 21830 21697 65292 24182 32473 20986 20027 22270 21644 26631 39064 20248 21270 24314 35758 12290"
    )

    expect(
      getStudentNextActionGroup(
        {
          id: "assistant-product-recommendation",
          role: "assistant",
          content: productRecommendation,
        },
        productRecommendation
      )
    ).toBe("productFollowup")
  })

  it.each([
    {
      expected: "productFollowup",
      message: textFromCodeString(
        "20808 22260 32469 38450 26194 34915 30340 20027 22270 12289 26631 39064 12289 20215 26684 21644 22823 23398 29983 20154 32676 20570 23567 33539 22260 27979 35797 12290"
      ),
      scenario: "product optimization",
      user: textFromCodeString(
        "25105 24819 21334 19968 27454 38450 26194 34915 65292 38754 21521 22823 23398 29983 65292 20215 26684 49 50 57 20803 65292 20027 22270 28857 20987 19981 38169 20294 21152 36141 36716 21270 19968 33324 65292 24590 20040 20248 21270 65311"
      ),
    },
    {
      expected: "productFollowup",
      message: textFromCodeString(
        "21487 20197 28151 21512 35775 38382 33258 24049 30340 39038 23458 12289 31454 21697 39038 23458 21644 23578 26410 21521 20219 20309 19968 26041 36141 20080 30340 20154 65292 20943 23569 26679 26412 20559 24046 12290"
      ),
      scenario: "market research",
      user: textFromCodeString(
        "20934 22791 39564 35777 19968 27454 26032 21830 21697 30340 24066 22330 38656 27714 65292 24212 22914 20309 32452 21512 33258 24049 30340 39038 23458 12289 31454 21697 39038 23458 21644 26410 36141 20080 20154 32676 65311"
      ),
    },
    {
      expected: "service",
      message: textFromCodeString(
        "20808 26680 23454 35746 21333 21644 29289 27969 29366 24577 65292 20877 28165 26970 35828 26126 22788 29702 26102 38480 12289 36864 27454 25110 34917 21457 26041 26696 12290"
      ),
      scenario: "customer service",
      user: textFromCodeString(
        "39038 23458 36141 20080 21830 21697 21518 21453 39304 37197 36865 24310 36831 65292 23458 26381 24212 35813 24590 26679 22788 29702 25237 35785 24182 21578 30693 34917 25937 26041 26696 65311"
      ),
    },
  ])(
    "keeps $scenario out of the GA4 conversion override",
    ({ expected, message, user }) => {
      expect(
        getStudentNextActionGroup(
          { id: `assistant-${expected}`, role: "assistant", content: message },
          user
        )
      ).toBe(expected)
    }
  )
})
